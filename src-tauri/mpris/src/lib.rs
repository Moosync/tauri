use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    time::Duration,
};

pub use souvlaki::{MediaControlEvent, SeekDirection};

use souvlaki::{MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use types::{
    errors::errors::Result,
    mpris::{MprisPlayerDetails, PlaybackState},
};

pub struct MprisHolder {
    controls: Mutex<MediaControls>,
    pub event_rx: Arc<Mutex<Receiver<MediaControlEvent>>>,
    last_duration: Mutex<u64>,
    last_state: Mutex<PlaybackState>,
}

impl MprisHolder {
    pub fn new() -> Result<MprisHolder> {
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = {
            use windows::Win32::UI::Input::KeyboardAndMouse::GetActiveWindow;

            let console_window = unsafe { GetActiveWindow() };
            let hwnd = console_window as *mut c_void;
            Some(hwnd)
        };

        let config = PlatformConfig {
            display_name: "Moosync",
            dbus_name: "moosync",
            hwnd,
        };

        let mut controls = MediaControls::new(config)?;

        let (event_tx, event_rx) = mpsc::channel();
        controls.attach(move |event| {
            event_tx.send(event).unwrap();
        })?;

        #[cfg(target_os = "windows")]
        thread::spawn(|| {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));

                // this must be run repeatedly by your program to ensure
                // the Windows event queue is processed by your application
                windows::pump_event_queue();
            }
        });

        Ok(MprisHolder {
            controls: Mutex::new(controls),
            event_rx: Arc::new(Mutex::new(event_rx)),
            last_duration: Mutex::new(0),
            last_state: Mutex::new(PlaybackState::STOPPED),
        })
    }

    pub fn set_metadata(&self, metadata: MprisPlayerDetails) -> Result<()> {
        let mut controls = self.controls.lock().unwrap();
        let duration = metadata.duration.map(|d| (d * 1000.0) as u64);
        controls.set_metadata(MediaMetadata {
            title: metadata.title.as_deref(),
            album: metadata.album_name.as_deref(),
            artist: metadata.artist_name.as_deref(),
            cover_url: metadata.thumbnail.as_deref(),
            duration: duration.map(Duration::from_millis),
        })?;

        Ok(())
    }

    pub fn set_playback_state(&self, state: PlaybackState) -> Result<()> {
        let last_duration = self.last_duration.lock().unwrap();
        let parsed = match state {
            PlaybackState::PLAYING => MediaPlayback::Playing {
                progress: Some(MediaPosition(Duration::from_millis(
                    last_duration.to_owned(),
                ))),
            },
            PlaybackState::PAUSED | PlaybackState::LOADING => MediaPlayback::Paused {
                progress: Some(MediaPosition(Duration::from_millis(
                    last_duration.to_owned(),
                ))),
            },
            PlaybackState::STOPPED => MediaPlayback::Stopped,
        };

        let mut controls = self.controls.lock().unwrap();
        controls.set_playback(parsed)?;
        drop(controls);

        let mut last_state = self.last_state.lock().unwrap();
        *last_state = state;
        Ok(())
    }

    pub fn set_position(&self, duration: f64) -> Result<()> {
        let mut last_duration = self.last_duration.lock().unwrap();
        *last_duration = (duration * 1000.0) as u64;
        drop(last_duration);

        let last_state = self.last_state.lock().unwrap().clone();
        self.set_playback_state(last_state)?;
        Ok(())
    }
}

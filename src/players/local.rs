use std::rc::Rc;

use leptos::{
    create_node_ref,
    ev::{ended, error, loadeddata, loadstart, pause, play, timeupdate},
    event_target,
    html::{audio, Audio, Div},
    spawn_local, HtmlElement, NodeRef,
};

use leptos_use::use_event_listener;
use tokio::sync::oneshot::Sender as OneShotSender;
use types::{errors::Result, songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen_futures::JsFuture;

use crate::utils::common::{convert_file_src, get_blob_url};

use super::generic::GenericPlayer;

macro_rules! listen_event {
    ($self:expr, $tx:expr, $event:ident, $handler:expr) => {{
        let unlisten = use_event_listener($self.node_ref, $event, move |evt| {
            let tx = $tx.clone();
            spawn_local(async move {
                let val = $handler(evt);
                let _ = tx(val);
                // if let Err(res) = res {
                //     console_log!("Error sending event: {:?}", res);
                // }
            });
        });
        $self.listeners.push(Rc::new(Box::new(unlisten)));
    }};
}

macro_rules! generate_event_listeners {
    ($($method:tt => $event:ident => $handler:expr),*) => {
        $(
            fn $method(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
                listen_event!(self, tx, $event, $handler);
            }
        )*
    };
}

#[derive(Clone)]
pub struct LocalPlayer {
    pub audio_element: HtmlElement<Audio>,
    node_ref: NodeRef<Audio>,
    listeners: Vec<Rc<Box<dyn Fn()>>>,
    event_tx: Option<Rc<Box<dyn Fn(PlayerEvents)>>>,
}

impl std::fmt::Debug for LocalPlayer {
    #[tracing::instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalPlayer")
            .field("audio_element", &self.audio_element.tag_name())
            .finish()
    }
}

impl LocalPlayer {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        let mut audio_element = audio();
        let node_ref = create_node_ref();

        audio_element = audio_element.node_ref(node_ref);

        LocalPlayer {
            audio_element,
            node_ref,
            listeners: vec![],
            event_tx: None,
        }
    }

    generate_event_listeners!(
        listen_onplay => play => |_| PlayerEvents::Play,
        listen_onpause => pause => |_| PlayerEvents::Pause,
        listen_onended => ended => |_| PlayerEvents::Ended,
        listen_onloadstart => loadstart => |_| PlayerEvents::Loading,
        listen_onloadend => loadeddata => |_| PlayerEvents::Play,
        listen_onerror => error => |err| PlayerEvents::Error(format!("{:?}", err).into()),
        listen_ontimeupdate => timeupdate => |evt|{
            let target = event_target::<leptos::web_sys::HtmlAudioElement>(&evt);
            let time = target.current_time();
            PlayerEvents::TimeUpdate(time)
        }
    );
}

impl GenericPlayer for LocalPlayer {
    #[tracing::instrument(level = "trace", skip(self, player_container))]
    fn initialize(&self, player_container: NodeRef<Div>) {
        let node_ref = self.node_ref;
        player_container.on_load(move |elem| {
            let audio_elem = node_ref.get().unwrap();
            if let Err(e) = elem.append_child(&audio_elem) {
                tracing::error!("Error initializing local player: {:?}", e);
            }
        });
        tracing::debug!("Returning from local player initialize")
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn key(&self) -> String {
        "local".into()
    }

    #[tracing::instrument(level = "trace", skip(self, src, resolver))]
    fn load(&self, src: String, resolver: OneShotSender<()>) {
        let mut src = convert_file_src(src);
        tracing::debug!("Loading audio {}", src);

        let audio_element = self.audio_element.clone();
        spawn_local(async move {
            if src.starts_with("asset") {
                src = get_blob_url(src).await;
            }

            audio_element.set_src(src.as_str());
            audio_element.load();

            resolver.send(()).expect("Load failed to resolve");
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn play(&self) -> Result<()> {
        let promise = self.audio_element.play()?;
        let event_tx = self.event_tx.clone();
        spawn_local(async move {
            if let Err(e) = JsFuture::from(promise).await {
                tracing::error!("Error playing audio: {:?}", e);
                if let Some(tx) = event_tx {
                    tx(PlayerEvents::Error(
                        format!("Error playing audio: {:?}", e).into(),
                    ));
                }
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn pause(&self) -> Result<()> {
        self.audio_element.pause()?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn provides(&self) -> &[SongType] {
        &[
            SongType::LOCAL,
            SongType::URL,
            SongType::YOUTUBE,
            SongType::SPOTIFY,
        ]
    }

    #[tracing::instrument(level = "trace", skip(self, volume))]
    fn set_volume(&self, volume: f64) -> Result<()> {
        self.audio_element.set_volume(volume / 100f64);
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn get_volume(&self) -> Result<f64> {
        Ok(self.audio_element.volume())
    }

    #[tracing::instrument(level = "trace", skip(self, tx))]
    fn add_listeners(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
        self.listen_onplay(tx.clone());
        self.listen_onpause(tx.clone());
        self.listen_onended(tx.clone());
        self.listen_onloadstart(tx.clone());
        self.listen_onloadend(tx.clone());
        self.listen_ontimeupdate(tx.clone());
        self.listen_onerror(tx.clone());
        self.event_tx = Some(tx)
    }

    #[tracing::instrument(level = "trace", skip(self, pos))]
    fn seek(&self, pos: f64) -> Result<()> {
        Ok(self.audio_element.fast_seek(pos)?)
    }

    #[tracing::instrument(level = "trace", skip(self, song))]
    fn can_play(&self, song: &types::songs::Song) -> bool {
        let playback_url = song
            .song
            .path
            .clone()
            .map(convert_file_src)
            .or(song.song.playback_url.clone());
        tracing::debug!("Checking playback url {:?}", playback_url);
        if let Some(playback_url) = playback_url {
            return playback_url.starts_with("http://")
                || playback_url.starts_with("https://")
                || playback_url.starts_with("asset");
        }

        false
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn stop(&mut self) -> Result<()> {
        self.pause()?;
        self.audio_element.set_src_object(None);

        for listener in self.listeners.iter() {
            listener();
        }
        self.listeners.clear();
        self.event_tx = None;
        Ok(())
    }
}

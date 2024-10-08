use std::{
    f64::consts::PI,
    fs,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, MutexGuard,
    },
    thread,
    time::Duration,
};

use rodio::{Decoder, OutputStream, Sink};
use stream_download::{
    http::{reqwest::Client, HttpStream},
    source::SourceStream,
    storage::temp::TempStorageProvider,
    Settings, StreamDownload,
};
use tracing::{error, info, trace};
use types::{errors::Result, ui::player_details::PlayerEvents};

pub struct RodioPlayer {
    tx: Sender<RodioCommand>,
    events_rx: Arc<Mutex<Receiver<PlayerEvents>>>,
}

enum RodioCommand {
    SetSrc(String),
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(u64),
}

impl RodioPlayer {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new(cache_dir: PathBuf) -> Self {
        let (events_tx, events_rx) = channel::<PlayerEvents>();
        let cache_dir = cache_dir.join("rodio");
        if !cache_dir.exists() {
            fs::create_dir(cache_dir.clone()).unwrap();
        }
        let tx = Self::initialize(events_tx, cache_dir);
        Self {
            tx,
            events_rx: Arc::new(Mutex::new(events_rx)),
        }
    }

    async fn set_src(cache_dir: PathBuf, src: String, sink: Arc<Sink>) -> Result<()> {
        if src.starts_with("http") {
            trace!("Creating stream");
            let reader = StreamDownload::new_http(
                src.parse().unwrap(),
                TempStorageProvider::new_in(cache_dir),
                Settings::default(),
            )
            .await?;
            trace!("stream created");

            let decoder = rodio::Decoder::new(reader)?;
            trace!("decoder created");
            sink.append(decoder);
            trace!("decoder appended");

            Ok(())
        } else {
            let path = PathBuf::from_str(src.as_str()).unwrap();
            if path.exists() {
                let file = fs::File::open(path)?;
                let reader = BufReader::new(file);
                let decoder = Decoder::new(reader)?;
                sink.append(decoder);
                return Ok(());
            }

            Err("Failed to read src".into())
        }
    }

    pub fn get_events_rx(&self) -> Arc<Mutex<Receiver<PlayerEvents>>> {
        self.events_rx.clone()
    }

    fn send_event(events_tx: Sender<PlayerEvents>, event: PlayerEvents) {
        events_tx.send(event).unwrap();
    }

    fn initialize(events_tx: Sender<PlayerEvents>, cache_dir: PathBuf) -> Sender<RodioCommand> {
        let (tx, rx) = channel::<RodioCommand>();
        thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Arc::new(rodio::Sink::try_new(&stream_handle).unwrap());

            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            let events_tx = events_tx.clone();
            runtime.block_on(async move {
                while let Ok(command) = rx.recv() {
                    let sink = sink.clone();

                    match command {
                        RodioCommand::SetSrc(src) => {
                            sink.clear();
                            Self::send_event(events_tx.clone(), PlayerEvents::Loading);
                            if let Err(err) = Self::set_src(cache_dir.clone(), src, sink).await {
                                error!("Failed to set src: {:?}", err);
                                Self::send_event(events_tx.clone(), PlayerEvents::Error(err))
                            } else {
                                info!("Set src");
                            }
                        }
                        RodioCommand::Play => {
                            if !sink.empty() {
                                sink.play();
                                Self::send_event(events_tx.clone(), PlayerEvents::Play)
                            }
                        }
                        RodioCommand::Pause => {
                            if !sink.empty() {
                                sink.pause();
                                Self::send_event(events_tx.clone(), PlayerEvents::Pause)
                            }
                        }
                        RodioCommand::Stop => {
                            if !sink.empty() {
                                sink.stop();
                                sink.clear();
                                Self::send_event(events_tx.clone(), PlayerEvents::Pause)
                            }
                        }
                        RodioCommand::SetVolume(volume) => {
                            if !sink.empty() {
                                sink.set_volume(volume);
                            }
                        }
                        RodioCommand::Seek(pos) => {
                            if !sink.empty() {
                                if let Err(err) = sink.try_seek(Duration::from_secs(pos)) {
                                    error!("Failed to seek: {:?}", err)
                                } else {
                                    Self::send_event(
                                        events_tx.clone(),
                                        PlayerEvents::TimeUpdate(pos as f64),
                                    )
                                }
                            }
                        }
                    }
                }
            });
        });

        tx
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_load(&self, src: String) -> Result<()> {
        info!("Loading src={}", src);
        self.tx.send(RodioCommand::SetSrc(src)).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_play(&self) -> Result<()> {
        self.tx.send(RodioCommand::Play).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_pause(&self) -> Result<()> {
        self.tx.send(RodioCommand::Pause).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_stop(&self) -> Result<()> {
        self.tx.send(RodioCommand::Stop).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, pos))]
    pub async fn rodio_seek(&self, pos: f64) -> Result<()> {
        self.tx
            .send(RodioCommand::Seek(pos.abs().round() as u64))
            .unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_set_volume(&self, volume: f32) -> Result<()> {
        self.tx.send(RodioCommand::SetVolume(volume)).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_get_volume(&self) -> Result<f32> {
        Ok(0f32)
    }
}

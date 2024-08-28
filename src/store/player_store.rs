use bitcode::{Decode, Encode};
use futures::lock::Mutex;
use indexed_db_futures::{
    request::{IdbOpenDbRequestLike, OpenDbRequest},
    IdbDatabase, IdbVersionChangeEvent,
};
use leptos::{create_effect, create_rw_signal, RwSignal, SignalGet, SignalSet, SignalUpdate};
use rand::seq::SliceRandom;
use serde::Serialize;
use std::{cmp::min, collections::HashMap, rc::Rc};
use types::{
    extensions::ExtensionExtraEvent,
    preferences::CheckboxPreference,
    songs::Song,
    ui::player_details::{PlayerState, RepeatModes, VolumeMode},
};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

use crate::{
    console_log,
    utils::{
        db_utils::{read_from_indexed_db, write_to_indexed_db},
        extensions::send_extension_event,
        mpris::{set_metadata, set_playback_state, set_position},
    },
};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Encode, Decode)]
pub struct Queue {
    pub song_queue: Vec<String>,
    pub current_index: usize,
    pub data: HashMap<String, Song>,
}

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct PlayerDetails {
    pub current_time: f64,
    pub force_seek: f64,
    pub state: PlayerState,
    pub has_repeated: bool,
    pub repeat: RepeatModes,
    volume: f64,
    volume_mode: VolumeMode,
    volume_map: HashMap<String, f64>,
    clamp_map: HashMap<String, f64>,
}

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct PlayerStoreData {
    pub queue: Queue,
    pub current_song: Option<Song>,
    pub player_details: PlayerDetails,
    pub player_blacklist: Vec<String>,
    pub force_load_song: bool,
}

#[derive(Debug)]
pub struct PlayerStore {
    pub data: PlayerStoreData,
    db: Rc<Mutex<Option<Rc<IdbDatabase>>>>,
}

impl PlayerStore {
    pub fn new() -> RwSignal<Self> {
        let db_rc = Rc::new(Mutex::new(None));
        let db_rc_clone = db_rc.clone();

        let signal = create_rw_signal(Self {
            data: PlayerStoreData::default(),
            db: db_rc,
        });
        spawn_local(async move {
            let mut db_req: OpenDbRequest = if let Ok(db_req) = IdbDatabase::open_u32("moosync", 1)
            {
                db_req
            } else {
                console_log!("Failed to get indexed db req");
                return;
            };
            db_req.set_on_upgrade_needed(Some(
                |evt: &IdbVersionChangeEvent| -> Result<(), JsValue> {
                    // Check if the object store exists; create it if it doesn't
                    if !evt.db().object_store_names().any(|n| n == "player_store") {
                        evt.db().create_object_store("player_store")?;
                    }
                    Ok(())
                },
            ));

            let db: Option<Rc<IdbDatabase>> = if let Ok(db) = db_req.await {
                Some(Rc::new(db))
            } else {
                console_log!("Failed to open indexed db");
                None
            };

            let mut db_rc = db_rc_clone.lock().await;
            *db_rc = db;
            drop(db_rc);

            let data_signal = create_rw_signal(None);
            Self::restore_store(db_rc_clone, data_signal);
            create_effect(move |_| {
                let data = data_signal.get();
                console_log!("Got data {:?}", data);
                signal.update(|s| {
                    if let Some(data) = data {
                        s.data = data;
                    }
                });
            });
        });

        signal
    }

    pub fn get_current_song(&self) -> Option<Song> {
        self.data.current_song.clone()
    }

    pub fn get_queue(&self) -> Queue {
        self.data.queue.clone()
    }

    pub fn get_player_state(&self) -> PlayerState {
        self.data.player_details.state
    }

    pub fn get_queue_len(&self) -> usize {
        self.data.queue.song_queue.len()
    }

    pub fn get_queue_index(&self) -> usize {
        self.data.queue.current_index
    }

    pub fn get_force_load(&self) -> bool {
        self.data.force_load_song
    }

    pub fn set_has_repeated(&mut self, has_repeated: bool) {
        self.data.player_details.has_repeated = has_repeated;
    }

    pub fn get_has_repeated(&self) -> bool {
        self.data.player_details.has_repeated
    }

    pub fn get_repeat(&self) -> RepeatModes {
        self.data.player_details.repeat
    }

    pub fn get_force_seek(&self) -> f64 {
        self.data.player_details.force_seek
    }

    pub fn get_current_time(&self) -> f64 {
        self.data.player_details.current_time
    }

    pub fn get_player_blacklist(&self) -> Vec<String> {
        self.data.player_blacklist.clone()
    }

    pub fn update_current_song(&mut self) {
        if self.data.queue.current_index >= self.data.queue.song_queue.len() {
            self.data.queue.current_index = 0;
        }
        let id = self.data.queue.song_queue[self.data.queue.current_index].clone();
        let song = self.data.queue.data.get(&id).cloned();

        if song == self.data.current_song && self.data.player_blacklist.is_empty() {
            return;
        }

        console_log!("Upading song in queue");
        self.data.current_song = song.clone();

        self.clear_blacklist();

        set_metadata(&song.clone().unwrap_or_default());
        send_extension_event(ExtensionExtraEvent::SongChanged([song]));

        self.dump_store();
    }

    pub fn add_to_queue(&mut self, songs: Vec<Song>) {
        self.add_to_queue_at_index(songs, self.data.queue.song_queue.len());
        self.update_current_song();
    }

    fn add_to_queue_at_index(&mut self, songs: Vec<Song>, index: usize) {
        let mut index = index;
        for song in songs {
            self.insert_song_at_index(song, index);
            index += 1;
        }
    }

    pub fn remove_from_queue(&mut self, index: usize) {
        self.data.queue.song_queue.remove(index);
        self.dump_store();
    }

    fn insert_song_at_index(&mut self, song: Song, index: usize) {
        let song_id = song.song._id.clone().unwrap();
        self.data.queue.data.insert(song_id.clone(), song);
        let insertion_index = min(self.data.queue.song_queue.len(), index);
        self.data.queue.song_queue.insert(insertion_index, song_id);

        self.dump_store();
    }

    pub fn play_now(&mut self, song: Song) {
        self.insert_song_at_index(song, self.data.queue.current_index + 1);
        self.data.queue.current_index += 1;
        self.update_current_song();
    }

    pub fn play_now_multiple(&mut self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        let first_song = songs.first();
        if let Some(first_song) = first_song {
            self.play_now(first_song.clone())
        }

        if songs.len() > 1 {
            self.add_to_queue_at_index(songs[1..].to_vec(), self.data.queue.current_index + 1);
        }
    }

    pub fn play_next(&mut self, song: Song) {
        self.insert_song_at_index(song, self.data.queue.current_index + 1);
    }

    pub fn play_next_multiple(&mut self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        let first_song = songs.first();
        if let Some(first_song) = first_song {
            self.play_next(first_song.clone())
        }

        if songs.len() > 1 {
            self.add_to_queue_at_index(songs[1..].to_vec(), self.data.queue.current_index + 1);
        }
    }

    pub fn change_index(&mut self, new_index: usize) {
        if new_index >= self.data.queue.song_queue.len() {
            return;
        }

        self.data.queue.current_index = new_index;
        self.update_current_song();
    }

    pub fn update_time(&mut self, new_time: f64) {
        self.data.player_details.current_time = new_time;
        set_position(new_time);
    }

    pub fn get_time(&self) -> f64 {
        self.data.player_details.current_time
    }

    pub fn force_seek_percent(&mut self, new_time: f64) {
        let new_time = if let Some(current_song) = &self.data.current_song {
            current_song.song.duration.unwrap_or_default() * new_time
        } else {
            0f64
        };

        console_log!("Got seek {}", new_time);
        self.data.player_details.force_seek = new_time;
        send_extension_event(ExtensionExtraEvent::Seeked([new_time]))
    }

    pub fn force_seek(&mut self, new_time: f64) {
        self.data.player_details.force_seek = new_time;
        send_extension_event(ExtensionExtraEvent::Seeked([new_time]))
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.data.player_details.state = state;

        set_playback_state(state);
        send_extension_event(ExtensionExtraEvent::PlayerStateChanged([state]))
    }

    fn get_song_key(&self) -> String {
        if let Some(current_song) = &self.data.current_song {
            return current_song
                .song
                .provider_extension
                .clone()
                .unwrap_or(current_song.song.type_.to_string());
        }
        "".to_string()
    }

    pub fn set_volume(&mut self, volume: f64) {
        if let VolumeMode::PersistSeparate = self.data.player_details.volume_mode {
            let song_key = self.get_song_key();
            if !song_key.is_empty() {
                console_log!("Setting volume for song: {}, {}", song_key, volume);
                self.data.player_details.volume_map.insert(song_key, volume);
            }
        }
        self.data.player_details.volume = volume;

        self.dump_store();
        send_extension_event(ExtensionExtraEvent::VolumeChanged([volume]))
    }

    pub fn get_volume(&self) -> f64 {
        let mut clamp = 100f64;
        let mut volume = self.data.player_details.volume;
        let song_key = self.get_song_key();
        if !song_key.is_empty() {
            if let VolumeMode::PersistSeparate = self.data.player_details.volume_mode {
                if let Some(current_volume) = self.data.player_details.volume_map.get(&song_key) {
                    volume = *current_volume;
                }
            }

            if let VolumeMode::PersistClamp = self.data.player_details.volume_mode {
                if let Some(current_clamp) = self.data.player_details.clamp_map.get(&song_key) {
                    clamp = *current_clamp;
                }
            }
        }
        let maxv = (clamp).ln();
        let scale = maxv / 100f64;
        let volume = volume.clamp(0f64, 100f64);
        if volume > 0f64 {
            return volume.ln() / scale;
        }
        volume
    }

    pub fn get_raw_volume(&self) -> f64 {
        if let VolumeMode::PersistSeparate = self.data.player_details.volume_mode {
            let song_key = self.get_song_key();
            if !song_key.is_empty() {
                if let Some(volume) = self.data.player_details.volume_map.get(&song_key) {
                    return *volume;
                }
            }
        }
        self.data.player_details.volume
    }

    pub fn get_queue_songs(&self) -> Vec<Song> {
        self.data
            .queue
            .song_queue
            .iter()
            .map(|index| {
                self.data
                    .queue
                    .data
                    .get(index)
                    .cloned()
                    .expect("Song does not exist in data")
            })
            .collect()
    }

    pub fn update_volume_mode(&mut self, mode: Vec<CheckboxPreference>) {
        for m in mode {
            if m.enabled {
                self.data.player_details.volume_mode = match m.key.as_str() {
                    "persist_separate" => VolumeMode::PersistSeparate,
                    "persist_clamp" => VolumeMode::PersistClamp,
                    _ => VolumeMode::Normal,
                };
                return;
            }
        }

        self.data.player_details.volume_mode = VolumeMode::Normal;
        self.dump_store();
    }

    pub fn next_song(&mut self) {
        self.data.queue.current_index += 1;
        if self.data.queue.current_index >= self.data.queue.song_queue.len() {
            self.data.queue.current_index = 0;
        }
        self.update_current_song();
    }

    pub fn prev_song(&mut self) {
        if self.data.queue.current_index == 0 {
            self.data.queue.current_index = self.data.queue.song_queue.len() - 1;
        } else {
            self.data.queue.current_index -= 1;
        }
        self.update_current_song();
    }

    pub fn toggle_repeat(&mut self) {
        let new_mode = match self.data.player_details.repeat {
            RepeatModes::None => RepeatModes::Once,
            RepeatModes::Once => RepeatModes::Loop,
            RepeatModes::Loop => RepeatModes::None,
        };

        self.data.player_details.repeat = new_mode;
        self.dump_store();
    }

    pub fn shuffle_queue(&mut self) {
        let binding = self.data.queue.song_queue.clone();
        let current_song = binding.get(self.data.queue.current_index).unwrap();
        let mut rng = rand::thread_rng();
        self.data.queue.song_queue.shuffle(&mut rng);
        let new_index = self
            .data
            .queue
            .song_queue
            .iter()
            .position(|v| v == current_song)
            .unwrap();
        self.data.queue.current_index = new_index;
        self.dump_store();
    }

    pub fn clear_queue(&mut self) {
        self.data.queue.song_queue.clear();
        self.data.queue.current_index = 0;
        self.update_current_song();
    }

    pub fn blacklist_player(&mut self, key: String) {
        self.data.player_blacklist.push(key);
        self.data.force_load_song = !self.data.force_load_song
    }

    fn clear_blacklist(&mut self) {
        self.data.player_blacklist.clear();
    }

    fn dump_store(&self) {
        let serialized = bitcode::encode(&self.data);
        let db = self.db.clone();
        spawn_local(async move {
            if let Err(e) =
                write_to_indexed_db(db, "player_store", "dump", &serialized.into()).await
            {
                console_log!("Failed to dump store: {:?}", e);
            }
        });
    }

    fn restore_store(
        db: Rc<Mutex<Option<Rc<IdbDatabase>>>>,
        signal: RwSignal<Option<PlayerStoreData>>,
    ) {
        spawn_local(async move {
            let bytes = read_from_indexed_db(db, "player_store", "dump").await;
            if let Ok(Some(bytes)) = bytes {
                let bytes = js_sys::Uint8Array::new(&bytes).to_vec();
                let deserialized = bitcode::decode::<'_, PlayerStoreData>(&bytes);
                if let Ok(deserialized) = deserialized {
                    console_log!("Got decoded {:?}", deserialized);
                    signal.set(Some(deserialized));
                }
            } else {
                console_log!("Error reading from indexed db {:?}", bytes);
            }
        });
    }
}

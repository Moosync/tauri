use bitcode::{Decode, Encode};
use indexed_db_futures::{database::Database, prelude::*};
use leptos::prelude::*;
use rand::seq::SliceRandom;
use serde::Serialize;
use std::{cmp::min, collections::HashMap};
use types::{
    preferences::CheckboxPreference,
    songs::Song,
    ui::extensions::ExtensionExtraEvent,
    ui::player_details::{PlayerState, RepeatModes, VolumeMode},
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    store::ui_store::UiStore,
    utils::{
        db_utils::{read_from_indexed_db, write_to_indexed_db},
        extensions::send_extension_event,
        mpris::{set_playback_state, set_position},
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
    old_volume: f64,
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
    scrobble_time: f64,
    scrobbled: bool,
    is_mobile: bool,
}

impl PlayerStore {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> RwSignal<Self> {
        // let db_rc = Arc::new(Mutex::new(None));
        // let db_rc_clone = db_rc.clone();

        let ui_store = expect_context::<RwSignal<UiStore>>();
        let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

        let player_store = Self {
            data: PlayerStoreData::default(),
            scrobble_time: 0f64,
            scrobbled: false,
            is_mobile,
        };

        tracing::debug!("Created player store {:?}", player_store);
        let signal = RwSignal::new(player_store);

        Self::load_state_from_idb(signal);

        signal
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_current_song(&self) -> Option<Song> {
        self.data.current_song.clone()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_queue(&self) -> Queue {
        self.data.queue.clone()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_player_state(&self) -> PlayerState {
        self.data.player_details.state
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_queue_len(&self) -> usize {
        self.data.queue.song_queue.len()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_queue_index(&self) -> usize {
        self.data.queue.current_index
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_force_load(&self) -> bool {
        self.data.force_load_song
    }

    #[tracing::instrument(level = "trace", skip(self, has_repeated))]
    pub fn set_has_repeated(&mut self, has_repeated: bool) {
        self.data.player_details.has_repeated = has_repeated;
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_has_repeated(&self) -> bool {
        self.data.player_details.has_repeated
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_repeat(&self) -> RepeatModes {
        self.data.player_details.repeat
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_force_seek(&self) -> f64 {
        self.data.player_details.force_seek
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_current_time(&self) -> f64 {
        self.data.player_details.current_time
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_player_blacklist(&self) -> Vec<String> {
        self.data.player_blacklist.clone()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update_current_song(&mut self, force: bool) {
        if self.data.queue.current_index >= self.data.queue.song_queue.len() {
            self.data.queue.current_index = 0;
        }
        let id = self
            .data
            .queue
            .song_queue
            .get(self.data.queue.current_index)
            .cloned()
            .unwrap_or_default();

        let song = self.data.queue.data.get(&id).cloned();

        if !force && song == self.data.current_song && self.data.player_blacklist.is_empty() {
            return;
        }

        tracing::debug!("Upading song in queue");
        self.data.current_song = song.clone();
        if self.data.current_song.is_none() {
            self.data.player_details.current_time = 0f64;
        }

        self.clear_blacklist();

        if force {
            self.data.force_load_song = !self.data.force_load_song;
        }

        self.scrobble_time = 0f64;
        self.scrobbled = false;

        self.dump_store();
    }

    #[tracing::instrument(level = "trace", skip(self, songs))]
    pub fn add_to_queue(&mut self, songs: Vec<Song>) {
        self.add_to_queue_at_index(songs, self.data.queue.song_queue.len());
        self.update_current_song(false);
    }

    #[tracing::instrument(level = "trace", skip(self, songs, index))]
    fn add_to_queue_at_index(&mut self, songs: Vec<Song>, index: usize) {
        let mut index = index;
        for song in songs {
            self.insert_song_at_index(song, index);
            index += 1;
        }
    }

    #[tracing::instrument(level = "trace", skip(self, index))]
    pub fn remove_from_queue(&mut self, index: usize) {
        self.data.queue.song_queue.remove(index);
        self.dump_store();
    }

    #[tracing::instrument(level = "trace", skip(self, song, index))]
    fn insert_song_at_index(&mut self, song: Song, index: usize) {
        let song_id = song.song._id.clone().unwrap();
        self.data.queue.data.insert(song_id.clone(), song);
        let insertion_index = min(self.data.queue.song_queue.len(), index);
        self.data.queue.song_queue.insert(insertion_index, song_id);

        self.dump_store();
    }

    #[tracing::instrument(level = "trace", skip(self, song))]
    pub fn play_now(&mut self, song: Song) {
        self.set_state(PlayerState::Playing);
        self.insert_song_at_index(song, self.data.queue.current_index + 1);
        self.data.queue.current_index += 1;
        self.update_current_song(true);
    }

    #[tracing::instrument(level = "trace", skip(self, songs))]
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

    #[tracing::instrument(level = "trace", skip(self, song))]
    pub fn play_next(&mut self, song: Song) {
        self.insert_song_at_index(song, self.data.queue.current_index + 1);
    }

    #[tracing::instrument(level = "trace", skip(self, songs))]
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

    #[tracing::instrument(level = "trace", skip(self, new_index))]
    pub fn change_index(&mut self, new_index: usize, force: bool) {
        self.data.queue.current_index = new_index;
        self.update_current_song(force);
    }

    #[tracing::instrument(level = "trace", skip(self, new_time))]
    pub fn update_time(&mut self, new_time: f64) {
        self.scrobble_time += 0f64.max(new_time - self.data.player_details.current_time);
        self.data.player_details.current_time = new_time;

        if self.scrobble_time > 20f64 && !self.scrobbled {
            if let Some(current_song) = self.get_current_song() {
                self.scrobbled = true;
                send_extension_event(ExtensionExtraEvent::Scrobble([current_song]));
            }
        }

        set_position(new_time);
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_time(&self) -> f64 {
        self.data.player_details.current_time
    }

    #[tracing::instrument(level = "trace", skip(self, new_time))]
    pub fn force_seek_percent(&mut self, new_time: f64) {
        let new_time = if let Some(current_song) = &self.data.current_song {
            current_song.song.duration.unwrap_or_default() * new_time
        } else {
            0f64
        };

        tracing::debug!("Got seek {}", new_time);
        self.data.player_details.force_seek = new_time;
        send_extension_event(ExtensionExtraEvent::Seeked([new_time]))
    }

    #[tracing::instrument(level = "trace", skip(self, new_time))]
    pub fn force_seek(&mut self, new_time: f64) {
        self.data.player_details.force_seek = new_time;
        send_extension_event(ExtensionExtraEvent::Seeked([new_time]))
    }

    #[tracing::instrument(level = "trace", skip(self, state))]
    pub fn set_state(&mut self, state: PlayerState) {
        tracing::debug!("Setting player state {:?}", state);
        self.data.player_details.state = state;
        self.dump_store();

        set_playback_state(state);
        send_extension_event(ExtensionExtraEvent::PlayerStateChanged([state]))
    }

    #[tracing::instrument(level = "trace", skip(self))]
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

    #[tracing::instrument(level = "trace", skip(self, volume))]
    pub fn set_volume(&mut self, volume: f64) {
        if let VolumeMode::PersistSeparate = self.data.player_details.volume_mode {
            let song_key = self.get_song_key();
            if !song_key.is_empty() {
                tracing::debug!("Setting volume for song: {}, {}", song_key, volume);
                self.data.player_details.volume_map.insert(song_key, volume);
            }
        }
        self.data.player_details.volume = volume;

        self.dump_store();
        send_extension_event(ExtensionExtraEvent::VolumeChanged([volume]))
    }

    pub fn toggle_mute(&mut self) {
        if self.data.player_details.volume > 0f64 {
            self.data.player_details.old_volume = self.data.player_details.volume;
            self.set_volume(0f64);
        } else if self.data.player_details.old_volume > 0f64 {
            self.set_volume(self.data.player_details.old_volume);
        } else {
            self.set_volume(50f64);
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_volume(&self) -> f64 {
        if self.is_mobile {
            return 100f64;
        }

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

    #[tracing::instrument(level = "trace", skip(self))]
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

    #[tracing::instrument(level = "trace", skip(self))]
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

    #[tracing::instrument(level = "trace", skip(self, mode))]
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

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn next_song(&mut self) {
        self.data.queue.current_index += 1;
        if self.data.queue.current_index >= self.data.queue.song_queue.len() {
            self.data.queue.current_index = 0;
        }
        self.update_current_song(false);
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn prev_song(&mut self) {
        if self.data.queue.current_index == 0 {
            self.data.queue.current_index = self.data.queue.song_queue.len() - 1;
        } else {
            self.data.queue.current_index -= 1;
        }
        self.update_current_song(false);
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn toggle_repeat(&mut self) {
        let new_mode = match self.data.player_details.repeat {
            RepeatModes::None => RepeatModes::Once,
            RepeatModes::Once => RepeatModes::Loop,
            RepeatModes::Loop => RepeatModes::None,
        };

        self.data.player_details.repeat = new_mode;
        self.dump_store();
    }

    #[tracing::instrument(level = "trace", skip(self))]
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

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn clear_queue(&mut self) {
        self.data.queue.song_queue.clear();
        self.data.queue.current_index = 0;
        self.update_current_song(false);
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn clear_queue_except_current(&mut self) {
        let current_song = self.get_current_song();

        let only_one_song = self.get_queue().song_queue.len() == 1;
        self.data.queue.song_queue.clear();
        self.data.queue.current_index = 0;

        if !only_one_song {
            if let Some(current_song) = current_song {
                self.add_to_queue(vec![current_song]);
            }
        }
        self.update_current_song(false);
    }

    #[tracing::instrument(level = "trace", skip(self, key))]
    pub fn blacklist_player(&mut self, key: String) {
        if self.data.player_blacklist.contains(&key) {
            return;
        }
        self.data.player_blacklist.push(key);
        self.data.force_load_song = !self.data.force_load_song
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn clear_blacklist(&mut self) {
        self.data.player_blacklist.clear();
    }

    pub fn load_state_from_idb(signal: RwSignal<PlayerStore>) {
        spawn_local(async move {
            match Database::open("moosync")
                .with_on_upgrade_needed(move |evt, db| {
                    if db.object_store_names().any(|n| n == "player_store") {
                        db.create_object_store("player_store").build()?;
                    }
                    Ok(())
                })
                .await
            {
                Err(e) => {
                    tracing::error!("Failed to create object store: {:?}", e);
                }
                Ok(db) => {
                    let data_signal = RwSignal::new(None);
                    Self::restore_store(data_signal, db);
                    Effect::new(move || {
                        let data = data_signal.get();
                        signal.update(|s| {
                            if let Some(data) = data {
                                tracing::debug!("Restored player store data {:?}", data);
                                s.data = data;
                                s.data.player_details.current_time = 0f64;
                            }
                        });
                    });
                }
            }
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn dump_store(&self) {
        let serialized = bitcode::encode(&self.data);
        spawn_local(async move {
            let db = Database::open("moosync").build();
            if let Ok(db) = db {
                if let Ok(db) = db.await {
                    if let Err(e) =
                        write_to_indexed_db(db, "player_store", "dump", serialized).await
                    {
                        tracing::error!("Failed to dump store: {:?}", e);
                    }
                }
            }
        });
    }

    #[tracing::instrument(level = "trace", skip(db, signal))]
    fn restore_store(signal: RwSignal<Option<PlayerStoreData>>, db: Database) {
        spawn_local(async move {
            let bytes = read_from_indexed_db(db, "player_store", "dump").await;
            if let Ok(Some(bytes)) = bytes {
                let bytes = js_sys::Uint8Array::new(&bytes).to_vec();
                let deserialized = bitcode::decode::<'_, PlayerStoreData>(&bytes);
                if let Ok(deserialized) = deserialized {
                    signal.set(Some(PlayerStoreData {
                        player_blacklist: vec![],
                        force_load_song: false,
                        ..deserialized
                    }));
                }
            } else {
                tracing::error!("Error reading from indexed db {:?}", bytes);
            }
        });
    }
}

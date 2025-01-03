use std::rc::Rc;

use futures::lock::Mutex;
use indexed_db_futures::IdbDatabase;
use indexed_db_futures::IdbQuerySource;
use leptos::{spawn_local, SignalSet, SignalUpdate};
use serde_wasm_bindgen::from_value;
use types::entities::QueryableAlbum;
use types::entities::QueryableArtist;
use types::entities::QueryableGenre;
use types::{
    entities::{GetEntityOptions, QueryablePlaylist},
    songs::{GetSongOptions, Song},
};
use wasm_bindgen::JsValue;
use web_sys::DomException;
use web_sys::IdbTransactionMode;

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_songs_by_option(
    options: GetSongOptions,
    setter: impl SignalSet<Value = Vec<Song>> + 'static,
) {
    spawn_local(async move {
        let songs = super::invoke::get_songs_by_options(options).await.unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_songs_by_option(
    options: GetSongOptions,
    setter: impl SignalSet<Value = Vec<Song>> + 'static,
) {
    use types::{entities::QueryableArtist, songs::SongType};

    let mut songs = vec![];
    for i in 0..1000 {
        let mut song = Song::default();
        song.song._id = Some(format!("song_id_{}", i));
        song.song.title = Some(format!("hello world {}", i));
        song.song.song_cover_path_low = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        song.song.song_cover_path_high = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        song.artists = Some(vec![QueryableArtist {
            artist_name: Some("Test artist".to_string()),
            ..Default::default()
        }]);
        song.song.type_ = SongType::LOCAL;
        song.song.duration = Some(96f64);
        song.song.playback_url =
            Some("https://cdn.freesound.org/previews/728/728162_462105-lq.mp3".into());
        songs.push(song);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_playlists_by_option(
    options: QueryablePlaylist,
    setter: impl SignalSet<Value = Vec<QueryablePlaylist>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut playlist = QueryablePlaylist::default();
        playlist.playlist_id = Some(format!("playlist_id_{}", i));
        playlist.playlist_name = format!("Playlist {}", i);
        playlist.playlist_coverpath = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(playlist);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_artists_by_option(
    options: QueryableArtist,
    setter: impl SignalSet<Value = Vec<QueryableArtist>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut artist = QueryableArtist::default();
        artist.artist_id = Some(format!("artist_id_{}", i));
        artist.artist_name = Some(format!("Artist {}", i));
        artist.artist_coverpath = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(artist);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_albums_by_option(
    options: QueryableAlbum,
    setter: impl SignalSet<Value = Vec<QueryableAlbum>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut album = QueryableAlbum::default();
        album.album_id = Some(format!("album_id_{}", i));
        album.album_name = Some(format!("Album {}", i));
        album.album_coverpath_high = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(album);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_genres_by_option(
    options: QueryableGenre,
    setter: impl SignalSet<Value = Vec<QueryableGenre>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut album = QueryableGenre::default();
        album.genre_id = Some(format!("genre_id_{}", i));
        album.genre_name = Some(format!("Genre {}", i));
        songs.push(album);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(setter))]
#[cfg(not(feature = "mock"))]
pub fn get_playlists_local<T>(setter: T)
where
    T: SignalSet<Value = Vec<QueryablePlaylist>>
        + SignalUpdate<Value = Vec<QueryablePlaylist>>
        + 'static,
{
    spawn_local(async move {
        let songs = serde_wasm_bindgen::from_value(
            super::invoke::get_entity_by_options(GetEntityOptions {
                playlist: Some(QueryablePlaylist::default()),
                ..Default::default()
            })
            .await
            .unwrap(),
        )
        .unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_playlists_by_option<T>(options: QueryablePlaylist, setter: T)
where
    T: SignalSet<Value = Vec<QueryablePlaylist>>
        + SignalUpdate<Value = Vec<QueryablePlaylist>>
        + Copy
        + 'static,
{
    use std::{collections::HashMap, rc::Rc};

    use leptos::{create_rw_signal, expect_context, RwSignal};

    use crate::{store::provider_store::ProviderStore, utils::common::fetch_infinite};

    let provider_store = expect_context::<Rc<ProviderStore>>();
    let next_page_tokens: RwSignal<
        HashMap<String, Rc<Mutex<types::providers::generic::Pagination>>>,
    > = create_rw_signal(HashMap::new());

    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            playlist: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting playlists: {:?}", res);
            return;
        }
        let songs: Vec<QueryablePlaylist> = from_value(res.unwrap()).unwrap();
        setter.set(songs);

        tracing::debug!("provider keys {:?}", provider_store.get_provider_keys());
        for key in provider_store.get_provider_keys() {
            tracing::debug!("Fetching playlists from {}", key);
            spawn_local(async move {
                let mut should_fetch = true;
                while should_fetch {
                    let res = fetch_infinite!(key, fetch_user_playlists, setter, next_page_tokens,);
                    match res {
                        Err(e) => {
                            tracing::error!(
                                "Failed to fetch playlist content from {}: {:?}",
                                key,
                                e
                            );
                            should_fetch = false;
                        }
                        Ok(should_fetch_inner) => should_fetch = should_fetch_inner,
                    }
                }

                setter.update(|p| p.dedup_by(|a, b| a == b));
            });
        }
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_artists_by_option(
    options: QueryableArtist,
    setter: impl SignalSet<Value = Vec<QueryableArtist>> + 'static,
) {
    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            artist: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting artists: {:?}", res);
            return;
        }
        let songs: Vec<QueryableArtist> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_albums_by_option(
    options: QueryableAlbum,
    setter: impl SignalSet<Value = Vec<QueryableAlbum>> + 'static,
) {
    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            album: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting albums: {:?}", res);
            return;
        }
        let songs: Vec<QueryableAlbum> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_genres_by_option(
    options: QueryableGenre,
    setter: impl SignalSet<Value = Vec<QueryableGenre>> + 'static,
) {
    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            genre: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting genres: {:?}", res);
            return;
        }
        let songs: Vec<QueryableGenre> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(songs, refresh_cb))]
pub fn add_songs_to_library(songs: Vec<Song>, refresh_cb: Rc<Box<dyn Fn()>>) {
    spawn_local(async move {
        let res = super::invoke::insert_songs(songs).await;
        if res.is_err() {
            tracing::error!("Error adding songs: {:?}", res);
        } else {
            refresh_cb.as_ref()();
        }
    });
}

#[tracing::instrument(level = "trace", skip(songs, refresh_cb))]
pub fn remove_songs_from_library(songs: Vec<Song>, refresh_cb: Rc<Box<dyn Fn()>>) {
    spawn_local(async move {
        let res = super::invoke::remove_songs(
            songs
                .iter()
                .map(|s| s.song._id.clone().unwrap_or_default())
                .collect(),
        )
        .await;
        if res.is_err() {
            tracing::error!("Error removing songs: {:?}", res);
        } else {
            refresh_cb.as_ref()();
        }
    });
}

#[tracing::instrument(level = "trace", skip(id, songs))]
pub fn add_to_playlist(id: String, songs: Vec<Song>) {
    spawn_local(async move {
        let res = super::invoke::add_to_playlist(id, songs).await;
        if res.is_err() {
            tracing::error!("Error adding to playlist: {:?}", res);
        }
    });
}

#[tracing::instrument(level = "trace", skip(playlist))]
pub fn create_playlist(playlist: QueryablePlaylist, songs: Option<Vec<Song>>) {
    spawn_local(async move {
        let res = super::invoke::create_playlist(playlist).await;
        match res {
            Err(res) => {
                tracing::error!("Failed to create playlist: {:?}", res);
            }
            Ok(playlist_id) => {
                if let Some(songs) = songs {
                    let res = super::invoke::add_to_playlist(playlist_id, songs).await;
                    if let Err(e) = res {
                        tracing::error!("Failed to add songs to playlist: {:?}", e);
                    }
                }
            }
        }
    });
}

#[tracing::instrument(level = "trace", skip(playlist, refresh_cb))]
pub fn remove_playlist(playlist: QueryablePlaylist, refresh_cb: Rc<Box<dyn Fn()>>) {
    if playlist.playlist_id.is_none() {
        return;
    }

    spawn_local(async move {
        let res = super::invoke::remove_playlist(playlist.playlist_id.unwrap()).await;
        if let Err(res) = res {
            tracing::error!("Failed to remove playlist: {:?}", res);
        }
        refresh_cb.as_ref()();
    });
}

#[tracing::instrument(level = "trace", skip(playlist))]
pub fn export_playlist(playlist: QueryablePlaylist) {
    spawn_local(async move {
        let res = super::invoke::export_playlist(playlist.playlist_id.unwrap()).await;
        if let Err(res) = res {
            tracing::error!("Failed to export playlist: {:?}", res);
        }
    });
}

#[tracing::instrument(level = "trace", skip(db, store, key, value))]
pub async fn write_to_indexed_db(
    db: Rc<Mutex<Option<Rc<IdbDatabase>>>>,
    store: &str,
    key: &str,
    value: &JsValue,
) -> Result<(), DomException> {
    let db = db.lock().await.clone();
    if let Some(db) = db {
        let tx = db.transaction_on_one_with_mode(store, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(store)?;
        store.put_key_val_owned(key, value)?.await?;
        tracing::debug!("Wrote to indexed db");
    }
    Ok(())
}

#[tracing::instrument(level = "trace", skip(db, store, key))]
pub async fn read_from_indexed_db(
    db: Rc<Mutex<Option<Rc<IdbDatabase>>>>,
    store: &str,
    key: &str,
) -> Result<Option<JsValue>, DomException> {
    let db = db.lock().await.clone();
    if let Some(db) = db {
        let tx = db.transaction_on_one(store)?;
        let store = tx.object_store(store)?;
        let res = store.get_owned(key)?.await?;
        return Ok(res);
    }
    Ok(None)
}

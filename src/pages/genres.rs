use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::store::player_store::PlayerStore;
use crate::utils::db_utils::get_genres_by_option;
use crate::utils::db_utils::get_songs_by_option;
use leptos::{
    component, create_rw_signal, create_write_slice, expect_context, view, IntoView, RwSignal,
    SignalGet, SignalWith,
};
use leptos_router::use_query_map;
use rand::seq::SliceRandom;
use std::rc::Rc;
use types::entities::QueryableGenre;
use types::songs::{GetSongOptions, Song};
use types::ui::song_details::SongDetailIcons;

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn SingleGenre() -> impl IntoView {
    let params = use_query_map();
    let genre_id = params.with(|params| params.get("id").cloned()).unwrap();

    let songs = create_rw_signal(vec![]);
    let selected_songs = create_rw_signal(vec![]);

    get_songs_by_option(
        GetSongOptions {
            genre: Some(QueryableGenre {
                genre_id: Some(genre_id),
                ..Default::default()
            }),
            ..Default::default()
        },
        songs,
    );

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_songs_setter = create_write_slice(player_store, |p, song| p.play_now(song));
    let add_to_queue_setter = create_write_slice(player_store, |p, songs| p.add_to_queue(songs));

    let play_songs = move || {
        let selected_songs = selected_songs.get();
        let songs = songs.get();

        let selected_songs = if selected_songs.is_empty() {
            songs
        } else {
            selected_songs
                .into_iter()
                .map(|song_index| {
                    let song: &Song = songs.get(song_index).unwrap();
                    song.clone()
                })
                .collect()
        };

        let first_song = selected_songs.first();
        if let Some(first_song) = first_song {
            play_songs_setter.set(first_song.clone())
        }
        add_to_queue_setter.set(selected_songs[1..].to_vec());
    };

    let add_to_queue = move || {
        let selected_songs = selected_songs.get();
        let songs = songs.get();
        if selected_songs.is_empty() {
            add_to_queue_setter.set(songs.clone());
        } else {
            let selected_songs = selected_songs
                .into_iter()
                .map(|song_index| {
                    let song: &Song = songs.get(song_index).unwrap();
                    song.clone()
                })
                .collect();
            add_to_queue_setter.set(selected_songs);
        }
    };

    let random = move || {
        let songs = songs.get();
        let random_song = songs.choose(&mut rand::thread_rng()).unwrap();
        play_songs_setter.set(random_song.clone());
    };

    let icons = create_rw_signal(SongDetailIcons {
        play: Some(Rc::new(Box::new(play_songs))),
        add_to_queue: Some(Rc::new(Box::new(add_to_queue))),
        random: Some(Rc::new(Box::new(random))),
        ..Default::default()
    });

    view! { <SongView songs=songs icons=icons selected_songs=selected_songs /> }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn AllGenres() -> impl IntoView {
    let genres = create_rw_signal(vec![]);
    get_genres_by_option(QueryableGenre::default(), genres.write_only());

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">Albums</div>
                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <CardView
                        items=genres
                        redirect_root="/main/genre"
                        card_item=move |(_, item)| {
                            let genre_name = item.genre_name.clone().unwrap_or_default();
                            let genre_id = item.genre_id.clone().unwrap_or_default();
                            SimplifiedCardItem {
                                title: genre_name,
                                cover: None,
                                id: item.clone(),
                                icon: None,
                                context_menu: None,
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

use leptos::html::Div;
use leptos::{
    component, create_effect, create_node_ref, create_read_slice, create_rw_signal,
    create_write_slice, expect_context, spawn_local, view, IntoView, NodeRef, RwSignal, Signal,
    SignalGet, SignalSet, SignalUpdate,
};
use leptos_virtual_scroller::VirtualScroller;
use serde::Serialize;
use types::songs::{Song, SongType};
use types::ui::player_details::PlayerState;
use types::ui::song_details::SongDetailIcons;
use web_sys::{ScrollBehavior, ScrollToOptions};

use crate::components::audiostream::AudioStream;
use crate::store::ui_store::UiStore;
use crate::utils::common::{get_high_img, invoke};
use crate::{
    components::{low_img::LowImg, provider_icon::ProviderIcon, songdetails::SongDetails},
    icons::{cross_icon::CrossIcon, trash_icon::TrashIcon},
    store::player_store::PlayerStore,
    utils::common::get_low_img,
};

#[tracing::instrument(
    level = "trace",
    skip(
        song,
        index,
        current_song_index,
        eq_playing,
        play_now,
        remove_from_queue
    )
)]
#[component]
pub fn QueueItem<T, D, P>(
    #[prop()] song: Song,
    index: usize,
    current_song_index: T,
    eq_playing: D,
    play_now: P,
    remove_from_queue: P,
) -> impl IntoView
where
    T: SignalGet<Value = usize> + Copy + 'static,
    D: SignalGet<Value = bool> + 'static,
    P: SignalSet<Value = usize> + 'static,
{
    view! {
        <div class="container-fluid item-container">
            <div class="row item-row">
                <LowImg
                    cover_img=get_low_img(&song)
                    play_now=move || play_now.set(index)
                    show_eq=move || index == current_song_index.get()
                    eq_playing=move || eq_playing.get()
                />
                <div class="col-lg-7 col-xl-8 col-5">
                    <div class="d-flex">
                        <div class="text-left song-title text-truncate">
                            {song.song.title.clone()}
                        </div>
                        {move || {
                            let extension = song.song.provider_extension.clone();
                            if let Some(extension) = extension {
                                view! { <ProviderIcon extension=extension /> }.into_view()
                            } else {
                                view! {}.into_view()
                            }
                        }}
                    </div>
                    <div class="text-left song-subtitle text-truncate">
                        {song
                            .artists
                            .unwrap_or_default()
                            .iter()
                            .map(|a| a.artist_name.clone().unwrap_or_default())
                            .collect::<Vec<String>>()
                            .join(", ")}
                    </div>

                </div>
                <div class="col-auto text-right ml-auto d-flex align-items-center">
                    <div class="ml-auto remove-button">
                        <TrashIcon on:click=move |_| remove_from_queue.set(index) />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip(show))]
#[component]
pub fn MusicInfo(#[prop()] show: Signal<bool>) -> impl IntoView {
    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let current_song = create_read_slice(player_store, move |p| p.get_current_song());
    let queue_songs = create_read_slice(player_store, move |p| p.get_queue_songs());
    let current_song_index = create_read_slice(player_store, |p| p.get_queue_index());
    let is_playing = create_read_slice(player_store, |p| {
        p.get_player_state() == PlayerState::Playing
    });
    let play_now = create_write_slice(player_store, |p, val| p.change_index(val));
    let remove_from_queue = create_write_slice(player_store, |p, val| p.remove_from_queue(val));

    let clear_queue = create_write_slice(player_store, |p, _| p.clear_queue_except_current());

    let ui_store = expect_context::<RwSignal<UiStore>>();

    create_effect(move |_| {
        let current_song = current_song.get();
        if let Some(current_song) = current_song {
            if current_song.song.type_ == SongType::SPOTIFY
                && current_song.song.playback_url.is_some()
            {
                spawn_local(async move {
                    #[derive(Serialize)]
                    struct GetCanvasArgs {
                        uri: String,
                    }

                    let res = invoke(
                        "get_canvaz",
                        serde_wasm_bindgen::to_value(&GetCanvasArgs {
                            uri: current_song.song.playback_url.clone().unwrap(),
                        })
                        .unwrap(),
                    )
                    .await;
                    if let Ok(res) = res {
                        tracing::debug!("Got canvas: {:?}", res);
                    } else {
                        tracing::error!("Failed to get canvaz {:?}", res)
                    }
                });
            }
        }
    });

    let scroller_ref: NodeRef<Div> = create_node_ref();
    create_effect(move |_| {
        let current_song_index = current_song_index.get();
        if let Some(el) = scroller_ref.get_untracked() {
            let el_top = 95usize * current_song_index;
            let options = ScrollToOptions::new();
            options.set_behavior(ScrollBehavior::Smooth);
            options.set_top(el_top as f64);
            el.scroll_with_scroll_to_options(&options);
        }
    });

    view! {
        <div
            class="slider"
            class:musicinfo-open=move || show.get()
            class:musicinfo-close=move || !show.get()
        >

            <div class="h-100 w-100">
                // Canvas
                <div class="dark-overlay" style="top: 0px;"></div>
                <img
                    class="bg-img"
                    src=move || {
                        if let Some(current_song) = current_song.get() {
                            get_high_img(&current_song)
                        } else {
                            String::new()
                        }
                    }
                />
                <div class="container-fluid w-100 h-100 music-info-container">
                    <div class="row no-gutters justify-content-end">
                        // Close button
                        <div class="col-auto">
                            <div class="cross-icon button-grow">
                                <CrossIcon on:click=move |_| {
                                    ui_store.update(|s| s.show_queue(false));
                                } />
                            </div>
                        </div>
                    </div>

                    <div class="row no-gutters justify-content-center h-100 flex-nowrap">
                        // Song details
                        <div class="col-4">
                            <div class="row no-gutters">
                                <div class="col position-relative">
                                    <AudioStream />
                                </div>
                            </div>
                            <SongDetails
                                icons=create_rw_signal(SongDetailIcons::default())
                                selected_song=current_song
                                show_lyrics=true
                            />
                        </div>
                        <div class="col-7 offset-1 right-container h-100">
                            <div class="h-100">
                                <div class="row">
                                    <div class="col-auto d-flex">
                                        <div class="rounded-btn">Save as playlist</div>
                                        <div
                                            class="rounded-btn"
                                            on:click=move |_| clear_queue.set(())
                                        >
                                            Clear
                                        </div>
                                    </div>
                                </div>
                                <div class="row queue-container-outer">
                                    <div class="col w-100 h-100 mr-4 queue-container">
                                        <div class="w-100 h-100">

                                            <VirtualScroller
                                                each=queue_songs
                                                item_height=95usize
                                                inner_el_style="width: calc(100% - 15px);"
                                                node_ref=scroller_ref
                                                children=move |(index, song)| {
                                                    view! {
                                                        <QueueItem
                                                            current_song_index=current_song_index
                                                            eq_playing=is_playing
                                                            song=song.clone()
                                                            index=index
                                                            play_now=play_now
                                                            remove_from_queue=remove_from_queue
                                                        />
                                                    }
                                                }
                                            />

                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}

use leptos::*;
use leptos::{component, view, IntoView, RwSignal, SignalGet, SignalSet};
use types::entities::QueryableArtist;
use types::ui::player_details::PlayerState;

use crate::components::low_img::LowImg;
use crate::components::musicinfo::MusicInfo;
use crate::icons::expand_icon::ExpandIcon;
use crate::icons::fav_icon::FavIcon;
use crate::icons::next_track_icon::NextTrackIcon;
use crate::icons::play_icon::PlayIcon;
use crate::icons::prev_track_icon::PrevTrackIcon;
use crate::icons::repeat_icon::RepeatIcon;
use crate::icons::shuffle_icon::ShuffleIcon;
use crate::icons::volume_icon::VolumeIcon;
use crate::store::player_store::PlayerStore;
use crate::store::ui_store::UiStore;
use crate::utils::common::{format_duration, get_low_img};

#[tracing::instrument(level = "trace", skip())]
#[component]
fn Details() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let current_song =
        create_read_slice(player_store, |player_store| player_store.get_current_song());

    let title = create_rw_signal("-".to_string());
    let artists_list = create_rw_signal::<Vec<QueryableArtist>>(vec![]);
    let cover_img = create_rw_signal("".to_string());

    create_effect(move |_| {
        let current_song = current_song.get().clone();
        if let Some(current_song) = &current_song {
            title.set(current_song.song.title.clone().unwrap());
            cover_img.set(get_low_img(current_song));

            if let Some(artists) = &current_song.artists {
                artists_list.set(artists.clone())
            }
            return;
        }
        title.set("-".into())
    });

    view! {
        <div class="row no-gutters align-items-center w-100">
            <div class="col-auto mr-3">

                {move || {
                    let cover_img = cover_img.get();
                    view! {
                        <LowImg
                            show_eq=|| false
                            eq_playing=|| false
                            cover_img=cover_img
                            show_play_button=false
                            play_now=|| {}
                        />
                    }
                }}

            </div>
            <div class="col text-truncate">
                <div class="row align-items-center justify-content-start">
                    <div class="col-auto w-100 d-flex">
                        <div title=move || title.get() class="text song-title text-truncate mr-2">
                            {move || title.get()}
                        </div>
                    </div>
                </div>

                <div class="row no-gutters">

                    {move || {
                        artists_list
                            .get()
                            .iter()
                            .map(|a| {
                                let artist_name = a.artist_name.clone().unwrap();
                                view! {
                                    <div class="col d-flex">
                                        <div
                                            class="text song-subtitle text-truncate"
                                            title=artist_name.clone()
                                        >
                                            {artist_name}
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}

                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn Controls() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let prev_track_dis = create_read_slice(player_store, |p| p.get_queue_len() <= 1);
    let next_track_dis = create_read_slice(player_store, |p| p.get_queue_len() <= 1);
    let is_play = create_read_slice(player_store, |p| {
        p.get_player_state() == PlayerState::Playing
    });
    let is_fav = create_rw_signal(false);
    let (repeat_mode, toggle_repeat) =
        create_slice(player_store, |p| p.get_repeat(), |p, _| p.toggle_repeat());
    let is_shuffle = create_rw_signal(true);
    let shuffle_queue = create_write_slice(player_store, |p, _| p.shuffle_queue());
    let current_time_sig =
        create_read_slice(player_store, |p| format_duration(p.get_current_time()));
    let total_duration_sig = create_read_slice(player_store, |p| {
        if let Some(current_song) = p.get_current_song() {
            format_duration(current_song.song.duration.unwrap_or(-1f64))
        } else {
            "00:00".to_string()
        }
    });

    let set_current_state = create_write_slice(player_store, |player, n| player.set_state(n));
    let next_song_setter = create_write_slice(player_store, |p, _| p.next_song());
    let prev_song_setter = create_write_slice(player_store, |p, _| p.prev_song());

    view! {
        <div class="row no-gutters align-items-center justify-content-center">
            <div class="col col-button">
                <PrevTrackIcon
                    disabled=prev_track_dis
                    on:click=move |_| {
                        if !prev_track_dis.get() {
                            prev_song_setter.set(())
                        }
                    }
                />
            </div>
            <div class="col col-button">
                <RepeatIcon mode=repeat_mode on:click=move |_| { toggle_repeat.set(()) } />
            </div>
            <div class="col col-play-button">
                <PlayIcon
                    play=is_play
                    on:click=move |_| {
                        let is_playing = is_play.get();
                        if is_playing {
                            set_current_state.set(PlayerState::Paused)
                        } else {
                            set_current_state.set(PlayerState::Playing)
                        }
                    }
                />

            </div>
            <div class="col col-button">
                <NextTrackIcon
                    disabled=next_track_dis
                    on:click=move |_| {
                        if !next_track_dis.get() {
                            next_song_setter.set(())
                        }
                    }
                />
            </div>
            <div class="col col-button">
                <ShuffleIcon
                    filled=is_shuffle.read_only()
                    on:click=move |_| {
                        shuffle_queue.set(());
                    }
                />
            </div>
            <div class="col col-button mr-1">
                <FavIcon filled=is_fav.read_only() />
            </div>
            <div class="col-md-3 col-5 align-self-center timestamp-container">
                <div class="row no-gutters align-items-center timestamp">
                    <div class="col-auto timestamp">{move || current_time_sig.get()}</div>
                    <div class="col-auto timestamp timestamp-extra ml-1">
                        / {move || total_duration_sig.get()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn ExtraControls() -> impl IntoView {
    let is_cut = create_rw_signal(false);

    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let (current_volume, set_current_volume) = create_slice(
        player_store,
        |player_store| player_store.get_raw_volume(),
        |player_store, volume| player_store.set_volume(volume),
    );

    let ui_store = expect_context::<RwSignal<UiStore>>();
    view! {
        <div class="row no-gutters align-items-center justify-content-end">
            <div class="col-auto volume-slider-container d-flex">
                <input
                    type="range"
                    min="0"
                    max="100"
                    class="volume-slider w-100 align-self-center"
                    prop:value=move || current_volume.get()
                    on:input=move |ev| {
                        set_current_volume.set(event_target_value(&ev).parse().unwrap())
                    }

                    id="myRange"
                    aria-label="volume"
                    style=move || {
                        format!(
                            "background: linear-gradient(90deg, var(--accent) 0%, var(--accent) {}%, var(--textSecondary) 0%);",
                            current_volume.get(),
                        )
                    }
                />

            </div>
            <div class="col-auto">
                <VolumeIcon cut=is_cut.read_only() />
            </div>
            <div class="col-auto expand-icon ml-3">
                <ExpandIcon on:click=move |_| { ui_store.update(move |s| s.toggle_show_queue()) } />
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn Slider() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let slider_process: NodeRef<html::Div> = create_node_ref();
    let offset_width = create_rw_signal(0f64);

    slider_process.on_load(move |s| {
        offset_width.set(s.offset_width() as f64);
    });
    let (current_time, set_current_time) = create_slice(
        player_store,
        |p| p.get_current_time(),
        move |p, val: f64| {
            p.force_seek_percent(
                val / slider_process.get_untracked().unwrap().offset_width() as f64,
            )
        },
    );

    let current_song = create_read_slice(player_store, |p| p.get_current_song());
    let total_time = create_rw_signal(1f64);

    create_effect(move |_| {
        let current_song = current_song.get();
        if let Some(current_song) = current_song {
            if let Some(duration) = current_song.song.duration {
                total_time.set(duration);
            }
        }
    });

    view! {
        <div class="timeline pl-2 pr-2">
            <div
                class="time-slider time-slider-ltr timeline pl-2 pr-2 timeline pl-2 pr-2"
                style="padding: 5px 0px; width: auto; height: 4px;"
            >
                <div
                    class="time-slider-rail"
                    ref=slider_process
                    on:click=move |ev| {
                        tracing::debug!("offset {}", ev.offset_x());
                        set_current_time.set(ev.offset_x() as f64);
                    }
                >

                    <div class="time-slider-bg">
                        <div
                            class="time-slider-process"
                            style=move || {
                                format!(
                                    "height: 100%; top: 0px; left: 0%; width: {}%; transition-property: width, left; transition-duration: 0.1s;",
                                    (current_time.get() / total_time.get()) * 100f64,
                                )
                            }
                        ></div>
                        <div
                            class="time-slider-dot"
                            role="slider"
                            tabindex="0"
                            style=move || {
                                format!(
                                    "width: 10px; height: 10px; transform: translate(-50%, -50%); top: 50%; left: {}%; transition: left 0.1s ease 0s;",
                                    (current_time.get() / total_time.get()) * 100f64,
                                )
                            }
                        >

                            <div class="time-slider-dot-handle"></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace")]
#[component]
pub fn MusicBar() -> impl IntoView {
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let show_musicinfo = create_read_slice(ui_store, move |s| s.get_show_queue());
    view! {
        <div class="musicbar-content d-flex">
            <MusicInfo show=show_musicinfo />
            <div class="background w-100">
                <div class="musicbar h-100">
                    <Slider />
                    <div class="container-fluid d-flex bar-container h-100 pb-2">
                        <div class="row no-gutters align-items-center justify-content-center align-content-center no-gutters w-100 control-row justify-content-between">
                            <div class="col-4 no-gutters details-col w-100">
                                <Details />
                            </div>

                            <div class="col align-self-center no-gutters controls-col">
                                <Controls />
                            </div>
                            <div class="col-lg-auto col-1 align-self-center no-gutters extra-col">
                                <ExtraControls />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}

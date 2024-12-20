use std::{collections::HashMap, rc::Rc};

use crate::components::cardview::{CardView, SimplifiedCardItem};
use colors_transform::{Color, Rgb};
use leptos::{
    component, create_effect, create_node_ref, create_rw_signal, ev::wheel, expect_context,
    html::Div, view, For, IntoView, Params, RwSignal, Show, SignalGet, SignalSet, SignalUpdate,
    SignalWith,
};
use leptos_router::{use_query, Params};
use leptos_use::{use_event_listener, use_resize_observer};
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

use crate::{
    components::songlist::SongList,
    icons::{next_icon::NextIcon, prev_icon::PrevIcon},
    store::provider_store::ProviderStore,
};

#[derive(Params, PartialEq)]
struct SearchQuery {
    q: Option<String>,
}

#[tracing::instrument(level = "trace", skip(keys, selected, single_select))]
#[component()]
pub fn TabCarousel(
    #[prop()] keys: Vec<String>,
    #[prop()] selected: RwSignal<Vec<String>>,
    #[prop()] single_select: bool,
    #[prop(optional, default = true)] align_left: bool,
) -> impl IntoView {
    let provider_container = create_node_ref::<Div>();

    let container_size = create_rw_signal(0f64);

    let show_next_icon = create_rw_signal(false);
    let show_prev_icon = create_rw_signal(false);

    let gradient_style = create_rw_signal("".to_string());

    let scroll_left = create_rw_signal(0);

    use_resize_observer(provider_container, move |entries, _| {
        leptos::request_animation_frame(move || {
            let rect = entries[0].content_rect();
            container_size.set(rect.width());
        });
    });

    // I have no idea what the fuck is this supposed to do.... but it works
    // Designed this like an year ago and never added comments to it
    let _ = use_event_listener(provider_container, wheel, move |ev| {
        ev.stop_propagation();
        ev.prevent_default();

        let provider_container = provider_container.get().unwrap();

        let scroll_left_prev = provider_container.scroll_left();
        if ev.delta_y() > 0f64 {
            provider_container.set_scroll_left(scroll_left_prev + 20);
        } else {
            provider_container.set_scroll_left(scroll_left_prev - 20);
        }

        scroll_left.set(provider_container.scroll_left());
    });

    create_effect(move |_| {
        let provider_container = provider_container.get();
        if let Some(provider_container) = provider_container {
            let scroll_width = provider_container.scroll_width();
            let scroll_left = scroll_left.get();
            let container_size = container_size.get() as i32;

            show_next_icon.set((scroll_left + container_size) < scroll_width);
            show_prev_icon.set(scroll_left > 0);

            let gradient_left = if show_prev_icon.get() { 10 } else { 0 };

            let gradient_right = if show_next_icon.get() { 90 } else { 100 };

            let primary_color = window()
                .unwrap()
                .get_computed_style(&provider_container)
                .unwrap()
                .unwrap()
                .get_property_value("--primary")
                .unwrap();

            let rgb_color = if primary_color.starts_with("#") {
                Rgb::from_hex_str(primary_color.as_str())
            } else {
                primary_color.parse::<Rgb>()
            }
            .unwrap();

            let rgba_string = format!(
                "rgba({}, {}, {}, 0)",
                rgb_color.get_red(),
                rgb_color.get_green(),
                rgb_color.get_blue()
            );

            gradient_style.set(format!(
                "background: linear-gradient(90deg, var(--primary) 0% , {} {}%, {} {}%, var(--primary) 100%);", rgba_string,
                gradient_left, rgba_string, gradient_right
            ));
        }
    });

    view! {
        <div class="container-fluid">
            <div class="row no-gutters">
                <div class="col song-header-options w-100">
                    <div class="row no-gutters align-items-center h-100">

                        <Show
                            when=move || { !show_prev_icon.get() }
                            fallback=|| {
                                view! {
                                    <div class="col-auto mr-3 h-100 d-flex align-items-center">
                                        <PrevIcon />
                                    </div>
                                }
                            }
                        >
                            <div></div>
                        </Show>

                        <div class="col provider-outer-container">
                            <div class="gradient-overlay" style=move || gradient_style.get()></div>

                            <div
                                node_ref=provider_container
                                class="provider-container d-flex"
                                class:justify-content-end=move || !align_left
                            >

                                <For
                                    each=move || keys.clone()
                                    key=|key| key.clone()
                                    children=move |key| {
                                        let key_tmp = key.clone();
                                        let key_tmp1 = key.clone();
                                        view! {
                                            <div
                                                class="h-100 item-checkbox-col mr-2"
                                                on:click=move |_| {
                                                    if selected.get().contains(&key_tmp) {
                                                        selected.update(|s| s.retain(|x| x != &key_tmp));
                                                    } else if !single_select {
                                                        selected.update(|s| s.push(key_tmp.clone()));
                                                    } else {
                                                        selected.set(vec![key_tmp.clone()]);
                                                    }
                                                }
                                            >
                                                <div
                                                    class="h-100 d-flex item-checkbox-container"
                                                    style=move || {
                                                        if selected.get().contains(&key_tmp1) {
                                                            "background: var(--textSecondary);"
                                                        } else {
                                                            "background: var(--secondary);"
                                                        }
                                                    }
                                                >
                                                    <span class="align-self-center provider-title">{key}</span>
                                                </div>
                                            </div>
                                        }
                                    }
                                />

                            </div>
                        </div>

                        <Show
                            when=move || { !show_next_icon.get() }
                            fallback=|| {
                                view! {
                                    <div class="col-auto ml-3 mr-3 h-100 d-flex align-items-center">
                                        <NextIcon />
                                    </div>
                                }
                            }
                        >
                            <div></div>
                        </Show>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn Search() -> impl IntoView {
    let query = use_query::<SearchQuery>();
    let term = move || {
        query.with(|query| {
            query
                .as_ref()
                .map(|query| query.q.clone())
                .unwrap_or_default()
        })
    };

    let search_results = create_rw_signal(HashMap::new());

    let provider_store = expect_context::<Rc<ProviderStore>>();
    let keys = provider_store.get_provider_keys();

    let selected_provider = create_rw_signal(vec![]);
    let selected_category = create_rw_signal(vec![]);

    let category_keys = vec![
        "Songs".to_string(),
        "Albums".to_string(),
        "Artists".to_string(),
        "Playlists".to_string(),
    ];

    if let Some(first_provider) = keys.first() {
        selected_provider.set(vec![first_provider.clone()]);
    }
    selected_category.set(vec![category_keys.first().unwrap().clone()]);

    let keys_clone = keys.clone();
    create_effect(move |_| {
        let search_term = term();
        if let Some(search_term) = search_term {
            if search_term.is_empty() {
                return;
            }
            tracing::debug!("Searching for: {}", search_term);

            let provider_store = provider_store.clone();

            let keys = keys_clone.clone();
            spawn_local(async move {
                for key in keys {
                    let res = provider_store
                        .provider_search(key.clone(), search_term.clone())
                        .await;
                    match res {
                        Ok(res) => {
                            search_results.update(|map| {
                                map.insert(key.clone(), res);
                            });
                        }
                        Err(err) => {
                            tracing::error!(
                                "Error searching for {} ({}): {:?}",
                                search_term,
                                key,
                                err
                            );
                        }
                    }
                }
            });
        }
    });

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid h-100 d-flex flex-column">

                <TabCarousel keys=keys.clone() selected=selected_provider single_select=true />
                <TabCarousel
                    keys=category_keys.clone()
                    selected=selected_category
                    single_select=true
                />

                <div class="container-fluid mt-3 search-song-list-container">
                    <div class="row no-gutters h-100">
                        {move || {
                            let search_results = search_results.get();
                            let binding = selected_provider.get();
                            let active_provider = binding.first();
                            if active_provider.is_none() {
                                return view! {}.into_view();
                            }
                            let active_provider = active_provider.unwrap();
                            if let Some(res) = search_results.get(active_provider) {
                                let binding = selected_category.get();
                                let active_category = binding.first();
                                if active_category.is_none() {
                                    return view! {}.into_view();
                                }
                                let active_category = active_category.unwrap();
                                return match active_category.as_str() {
                                    "Songs" => {
                                        view! {
                                            <div class="col h-100 song-list-compact">
                                                <SongList
                                                    hide_search_bar=true
                                                    song_list=create_rw_signal(res.songs.clone()).read_only()
                                                    selected_songs_sig=create_rw_signal(vec![])
                                                    filtered_selected=create_rw_signal(vec![])
                                                />
                                            </div>
                                        }
                                            .into_view()
                                    }
                                    "Albums" => {
                                        view! {
                                            <CardView
                                                items=create_rw_signal(res.albums.clone())
                                                redirect_root="/main/albums"
                                                card_item=move |(_, item)| {
                                                    SimplifiedCardItem {
                                                        title: item.album_name.clone().unwrap_or_default(),
                                                        cover: item.album_coverpath_high.clone(),
                                                        id: item.clone(),
                                                        icon: None,
                                                        context_menu: None,
                                                    }
                                                }
                                            />
                                        }
                                            .into_view()
                                    }
                                    "Artists" => {
                                        view! {
                                            <CardView
                                                items=create_rw_signal(res.artists.clone())
                                                redirect_root="/main/artists"
                                                card_item=move |(_, item)| {
                                                    SimplifiedCardItem {
                                                        title: item.artist_name.clone().unwrap_or_default(),
                                                        cover: item.artist_coverpath.clone(),
                                                        id: item.clone(),
                                                        icon: None,
                                                        context_menu: None,
                                                    }
                                                }
                                            />
                                        }
                                            .into_view()
                                    }
                                    "Playlists" => {
                                        view! {
                                            <CardView
                                                items=create_rw_signal(res.playlists.clone())
                                                redirect_root="/main/playlists/"
                                                card_item=move |(_, item)| {
                                                    SimplifiedCardItem {
                                                        title: item.playlist_name.clone(),
                                                        cover: item.playlist_coverpath.clone(),
                                                        id: item.clone(),
                                                        icon: None,
                                                        context_menu: None,
                                                    }
                                                }
                                            />
                                        }
                                            .into_view()
                                    }
                                    _ => view! {}.into_view(),
                                };
                            }
                            view! {}.into_view()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

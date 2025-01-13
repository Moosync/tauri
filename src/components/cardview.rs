use std::{hash::Hash, rc::Rc, sync::Arc};

use crate::{
    components::provider_icon::ProviderIcon,
    icons::{
        fav_playlist_icon::FavPlaylistIcon, play_hover_icon::PlayHoverIcon,
        song_default_icon::SongDefaultIcon,
    },
    store::ui_store::UiStore,
    utils::common::convert_file_src,
};
use leptos::{component, prelude::*, view, IntoView};
use leptos_router::{hooks::use_navigate, NavigateOptions};
use leptos_virtual_scroller::VirtualGridScroller;
use serde::Serialize;
use types::errors::MoosyncError;

type CardContextMenu<T> = Option<Arc<Box<dyn Fn(leptos::ev::MouseEvent, T)>>>;

#[derive(Clone)]
pub struct SimplifiedCardItem<T>
where
    T: Serialize + Send + Sync,
{
    pub title: String,
    pub cover: Option<String>,
    pub id: T,
    pub icon: Option<String>,
    pub context_menu: CardContextMenu<T>,
}

#[tracing::instrument(level = "trace", skip(item, songs_view, on_click))]
#[component()]
pub fn CardItem<T>(
    #[prop()] item: SimplifiedCardItem<T>,
    #[prop(optional, default = false)] songs_view: bool,
    #[prop(optional)] on_click: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    #[prop(optional)] is_mobile: bool,
) -> impl IntoView
where
    T: Serialize + Send + Sync,
{
    let show_default_icon = create_rw_signal(item.cover.is_none());

    let item_width = if is_mobile {
        (document().body().unwrap().client_width() / 2) - 30
    } else {
        200
    } as usize;

    view! {
        <div
            class="card mb-2 card-grow"
            style=move || {
                if !is_mobile {
                    "width: 200px;".to_string()
                } else {
                    format!("width: {}px;", item_width)
                }
            }
        >
            <div class="card-img-top">
                <div class="embed-responsive embed-responsive-1by1">
                    <div class="embed-responsive-item img-container">

                        <div class="card_overlay">
                            {move || {
                                let on_click = on_click.clone();
                                if songs_view {
                                    view! {
                                        <div
                                            class="play-button-song-list card-overlay-background d-flex justify-content-center w-100 h-100"
                                            on:click=move |_| {
                                                let on_click = on_click.clone();
                                                if let Some(cb) = on_click {
                                                    cb();
                                                }
                                            }
                                        >
                                            <PlayHoverIcon />
                                        </div>
                                    }
                                        .into_any()
                                } else {
                                    view! {}.into_any()
                                }
                            }}
                        </div>

                        <div class="provider-icon-overlay me-auto justify-content-center d-flex align-items-center">
                            {if let Some(icon) = item.icon.clone() {
                                view! { <ProviderIcon extension=icon /> }.into_any()
                            } else {
                                view! {}.into_any()
                            }}
                        </div>
                        {move || {
                            if show_default_icon.get() {
                                view! {
                                    <SongDefaultIcon class="rounded-corners img-fluid w-100 h-100"
                                        .into() />
                                }
                                    .into_any()
                            } else {
                                if let Some(cover) = item.cover.clone() {
                                    if cover == "favorites" {
                                        return view! {
                                            <FavPlaylistIcon class="rounded-corners img-fluid w-100 h-100" />
                                        }
                                            .into_any();
                                    }
                                }
                                view! {
                                    <img
                                        src=item.cover.clone().map(convert_file_src)
                                        class="rounded-corners img-fluid w-100 h-100"
                                        on:error=move |e| {
                                            tracing::error!(
                                                "Error loading cover image {:?}", MoosyncError::from(e.error())
                                            );
                                            show_default_icon.set(true);
                                        }
                                    />
                                }
                                    .into_any()
                            }
                        }}

                    </div>
                </div>
            </div>
            <div class="card-body">
                <p class="card-title text-truncate">{item.title}</p>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip(items, card_item, songs_view, on_click, key))]
#[component()]
pub fn CardView<T, S, C, K, KN>(
    #[prop()] items: S,
    #[prop()] key: KN,
    #[prop()] card_item: C,
    #[prop(optional, default = false)] songs_view: bool,
    #[prop(optional)] on_click: Option<Box<dyn Fn(T) + Send + Sync>>,
    #[prop(optional, default = "")] redirect_root: &'static str,
) -> impl IntoView
where
    T: 'static + Clone + Serialize + Send + Sync,
    C: Fn((usize, &T)) -> SimplifiedCardItem<T> + 'static + Send + Sync + Clone,
    KN: (Fn(&T) -> K) + 'static + Send + Sync + Clone,
    K: Eq + Hash + 'static,
    S: With<Value = Vec<T>> + Copy + 'static + Send + Sync,
{
    let on_click = on_click.map(Arc::new);

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    let item_width = if is_mobile {
        (document().body().unwrap().client_width() / 2) - 15
    } else {
        220
    } as usize;
    let item_height = if is_mobile {
        (document().body().unwrap().client_width() / 2) + 55
    } else {
        275
    } as usize;

    view! {
        <VirtualGridScroller
            each=items
            key=key
            item_width=item_width
            item_height=item_height
            children=move |data| {
                let data1 = data.1.clone();
                let data2 = data.1.clone();
                let card_item_data = card_item(data);
                let card_item_data1 = card_item_data.clone();
                let on_click = on_click.clone();
                if songs_view {
                    view! {
                        <CardItem
                            on:contextmenu=move |ev| {
                                ev.prevent_default();
                                if let Some(cb) = &card_item_data1.context_menu {
                                    cb(ev, data1.clone());
                                }
                            }
                            item=card_item_data
                            songs_view=songs_view
                            is_mobile=is_mobile
                            on_click=Arc::new(
                                Box::new(move || {
                                    if let Some(cb) = on_click.clone() {
                                        cb(data2.clone());
                                    }
                                }),
                            )
                        />
                    }
                        .into_any()
                } else {
                    let id = card_item_data.id.clone();
                    view! {
                        <div on:click=move |_| {
                            use_navigate()(
                                format!(
                                    "{}/single?entity={}",
                                    redirect_root,
                                    url_escape::encode_component(
                                        &serde_json::to_string(&id).unwrap(),
                                    ),
                                )
                                    .as_str(),
                                NavigateOptions::default(),
                            );
                        }>
                            <CardItem
                                on:contextmenu=move |ev| {
                                    ev.prevent_default();
                                    if let Some(cb) = &card_item_data1.context_menu {
                                        cb(ev, data1.clone());
                                    }
                                }
                                item=card_item_data
                                songs_view=songs_view
                                is_mobile=is_mobile
                            />
                        </div>
                    }
                        .into_any()
                }
            }
        />
    }
}

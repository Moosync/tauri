use std::rc::Rc;

use leptos::{
    component, create_rw_signal, create_write_slice, expect_context, view, IntoView, RwSignal,
    SignalUpdate,
};
use leptos_context_menu::ContextMenu;
use wasm_bindgen_futures::spawn_local;

use crate::{
    components::cardview::{CardView, SimplifiedCardItem},
    store::{player_store::PlayerStore, provider_store::ProviderStore},
    utils::{
        common::get_high_img, context_menu::SongItemContextMenu, db_utils::get_playlists_local,
    },
};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn Explore() -> impl IntoView {
    let provider_store: Rc<ProviderStore> = expect_context();
    let provider_keys = provider_store.get_provider_keys();
    let suggestion_items = create_rw_signal(vec![]);
    spawn_local(async move {
        for key in provider_keys {
            let suggestions = provider_store.get_suggestions(key).await;
            if let Ok(suggestions) = suggestions {
                suggestion_items.update(|s| s.extend(suggestions));
            }
        }
    });

    let player_store: RwSignal<PlayerStore> = expect_context();
    let play_now = create_write_slice(player_store, |p, s| p.play_now(s));

    let playlists = create_rw_signal(vec![]);
    get_playlists_local(playlists);

    let context_menu_data = SongItemContextMenu {
        current_song: None,
        song_list: suggestion_items.read_only(),
        selected_songs: create_rw_signal(vec![]),
        playlists,
    };
    let song_context_menu = Rc::new(ContextMenu::new(context_menu_data));

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">

                <div class="row page-title no-gutters">

                    <div class="col-auto">Suggestions</div>
                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >

                    <CardView
                        items=suggestion_items
                        songs_view=true
                        on_click=Box::new(move |item| { play_now.set(item) })
                        card_item=move |(_, item)| {
                            let song_context_menu = song_context_menu.clone();
                            SimplifiedCardItem {
                                title: item.song.title.clone().unwrap_or_default(),
                                cover: Some(get_high_img(item)),
                                id: item.clone(),
                                icon: item.song.provider_extension.clone(),
                                context_menu: Some(
                                    Rc::new(
                                        Box::new(move |ev, song| {
                                            ev.stop_propagation();
                                            let mut data = song_context_menu.get_data();
                                            data.current_song = Some(song.clone());
                                            drop(data);
                                            song_context_menu.show(ev);
                                        }),
                                    ),
                                ),
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}

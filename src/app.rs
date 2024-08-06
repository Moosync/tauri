use std::rc::Rc;

use crate::{
    components::prefs::static_components::SettingRoutes, console_log,
    players::librespot::LibrespotPlayer, utils::prefs::watch_preferences,
};
use leptos::{
    component, create_rw_signal, expect_context, provide_context, view, window,
    IntoView, RwSignal, SignalUpdate,
};
use leptos_i18n::provide_i18n_context;
use leptos_router::{Outlet, Redirect, Route, Router, Routes};
use types::preferences::CheckboxPreference;

use crate::{
    components::{
        musicbar::MusicBar,
        sidebar::{Sidebar, Tab},
        topbar::TopBar,
    },
    i18n::Locale,
    modals::modal_manager::ModalManager,
    pages::{
        albums::{AllAlbums, SingleAlbum},
        artists::{AllArtists, SingleArtist},
        genres::{AllGenres, SingleGenre},
        playlists::{AllPlaylists, SinglePlaylist},
        search::Search,
        songs::AllSongs,
    },
    store::{modal_store::ModalStore, player_store::PlayerStore, provider_store::ProviderStore},
};

#[component]
pub fn RedirectAll() -> impl IntoView {
    // TODO: Change to all songs
    view! { <Redirect path="/main" /> }
}

#[component]
fn CommonApp() -> impl IntoView {
    view! {
        <div>
            <MusicBar />
            <ModalManager />
            <Outlet />
        </div>
    }
}

#[component]
pub fn MainApp() -> impl IntoView {
    let tabs = vec![
        Tab::new("Queue", "Queue", ""),
        Tab::new("All Songs", "All Songs", "/main"),
        Tab::new("Playlists", "Playlists", "/main/playlists"),
        Tab::new("Artists", "Artists", "/main/artists"),
        Tab::new("Albums", "Albums", "/main/albums"),
        Tab::new("Genres", "Genres", "/main/genres"),
        Tab::new("Explore", "Explore", "/main/explore"),
    ];
    view! {
        <div>
            <TopBar />
            <Sidebar tabs=tabs />
            <div class="main-container">
                <Outlet />
            </div>
        </div>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(create_rw_signal(PlayerStore::new()));
    provide_context(Rc::new(ProviderStore::new()));
    provide_context(create_rw_signal(ModalStore::default()));

    provide_i18n_context::<Locale>();

    let unlisten = watch_preferences(|(key, value)| {
        console_log!("Preferences changed: {} = {:?}", key, value);
        if key == "prefs.volume_persist_mode" {
            let player_store = expect_context::<RwSignal<PlayerStore>>();
            player_store.update(|store| {
                store.update_volume_mode(serde_wasm_bindgen::from_value(value).unwrap())
            });
        } else if key == "prefs.spotify.enable" {
            let enabled: Vec<CheckboxPreference> = serde_wasm_bindgen::from_value(value).unwrap();
            for pref in enabled {
                if pref.key == "enable" {
                    LibrespotPlayer::set_enabled(pref.enabled)
                }
            }
        } else if key == "prefs.spotify.username" {
            let value = value.as_string().unwrap();
            LibrespotPlayer::set_has_username(!value.is_empty())
        } else if key == "prefs.spotify.password" {
            let value = value.as_string().unwrap();
            LibrespotPlayer::set_has_password(!value.is_empty())
        }
    });

    let window = window();
    if let Err(e) = window.add_event_listener_with_callback("beforeunload", &unlisten) {
        console_log!("Failed to set unmount hook: {:?}", e);
    }

    view! {
        <Router>
            <main id="app">
                <div class="appContainer">
                    <Routes>
                        <Route path="/" view=CommonApp>
                            <Route path="main" view=MainApp>
                                <Route path="" view=AllSongs />
                                <Route path="playlists" view=AllPlaylists />
                                <Route path="playlists/single" view=SinglePlaylist />
                                <Route path="artists" view=AllArtists />
                                <Route path="artists/single" view=SingleArtist />
                                <Route path="albums" view=AllAlbums />
                                <Route path="albums/single" view=SingleAlbum />
                                <Route path="genres" view=AllGenres />
                                <Route path="genres/single" view=SingleGenre />
                                <Route path="search" view=Search />
                            </Route>
                            <SettingRoutes />
                        </Route>
                        <Route path="*" view=RedirectAll />
                    </Routes>
                </div>
            </main>
        </Router>
    }
}

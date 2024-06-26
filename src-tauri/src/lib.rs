// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use extensions::get_extension_state;
use librespot::{
    get_canvaz, get_librespot_state, initialize_librespot, librespot_close, librespot_get_token,
    librespot_load, librespot_pause, librespot_play, librespot_seek, librespot_volume,
    register_event,
};
use logger::logger::{log_debug, log_error, log_info, log_warn};
use lyrics::{get_lyrics, get_lyrics_state};
use mpris::{get_mpris_state, set_metadata, set_playback_state, set_position};
use preference_holder::{
    get_preference_state, get_secure, initial, load_selective, load_selective_array,
    save_selective, set_secure,
};
use themes::{
    get_theme_handler_state, import_theme, load_all_themes, load_theme, remove_theme, save_theme,
    transform_css,
};

use extensions::{broadcast, download_extension, install_extension};
use scanner::{get_scanner_state, start_scan};
use tauri::{Manager, State};

use {
    db::{
        get_cache_state,
        {
            add_to_playlist, create_playlist, get_db_state, get_entity_by_options,
            get_songs_by_options, increment_play_count, increment_play_time, insert_songs,
            remove_from_playlist, remove_playlist, remove_songs, search_all, update_album,
            update_artist, update_lyrics, update_playlist, update_songs,
        },
    },
    oauth::handler::{get_oauth_state, OAuthHandler},
    window::handler::{
        close_window, disable_fullscreen, enable_fullscreen, get_platform, get_window_state,
        has_frame, is_maximized, maximize_window, minimize_window, open_external,
        open_file_browser, open_window, restart_app, toggle_dev_tools, toggle_fullscreen,
        update_zoom,
    },
    youtube::{get_playlist_content, get_video_url, get_youtube_scraper_state, search_yt},
};

use crate::oauth::handler::{register_oauth_path, unregister_oauth_path};

mod db;
mod extensions;
mod librespot;
mod logger;
mod lyrics;
mod mpris;
mod oauth;
mod preference_holder;
mod scanner;
mod themes;
mod window;
mod youtube;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let devtools = tauri_plugin_devtools::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(url) = argv.get(1) {
                let state: State<OAuthHandler> = app.state();
                state.handle_oauth(app.clone(), url.to_string()).unwrap();
            }
        }))
        .plugin(devtools)
        .invoke_handler(tauri::generate_handler![
            // Preferences
            save_selective,
            load_selective,
            load_selective_array,
            get_secure,
            set_secure,
            // Logger
            log_error,
            log_debug,
            log_info,
            log_warn,
            // DB
            insert_songs,
            remove_songs,
            get_songs_by_options,
            get_entity_by_options,
            search_all,
            create_playlist,
            add_to_playlist,
            remove_from_playlist,
            remove_playlist,
            update_album,
            update_artist,
            update_playlist,
            update_songs,
            update_lyrics,
            increment_play_count,
            increment_play_time,
            // OAuth
            register_oauth_path,
            unregister_oauth_path,
            // Window
            is_maximized,
            has_frame,
            close_window,
            get_platform,
            maximize_window,
            minimize_window,
            update_zoom,
            open_external,
            open_window,
            enable_fullscreen,
            disable_fullscreen,
            toggle_fullscreen,
            toggle_dev_tools,
            restart_app,
            open_file_browser,
            // Youtube
            search_yt,
            get_video_url,
            get_playlist_content,
            // Scanner
            start_scan,
            // Librespot
            initialize_librespot,
            librespot_play,
            librespot_pause,
            librespot_close,
            librespot_load,
            librespot_seek,
            librespot_volume,
            librespot_get_token,
            register_event,
            get_canvaz,
            // Themes
            load_all_themes,
            load_theme,
            save_theme,
            remove_theme,
            import_theme,
            transform_css,
            // MPRIS
            set_metadata,
            set_playback_state,
            set_position,
            // Lyrics
            get_lyrics,
            // Extensions
            broadcast,
            install_extension,
            download_extension,
        ])
        .setup(|app| {
            let db = get_db_state(app);
            app.manage(db);

            let cache = get_cache_state(app);
            app.manage(cache);

            let config = get_preference_state(app)?;
            app.manage(config);

            let oauth = get_oauth_state()?;
            app.manage(oauth);

            let window_state = get_window_state();
            app.manage(window_state);

            let yt_state = get_youtube_scraper_state();
            app.manage(yt_state);

            let scanner_state = get_scanner_state();
            app.manage(scanner_state);

            let librespot_state = get_librespot_state();
            app.manage(librespot_state);

            let theme_handler_state = get_theme_handler_state(app);
            app.manage(theme_handler_state);

            let mpris_state = get_mpris_state(app.app_handle().clone())?;
            app.manage(mpris_state);

            let lyrics_state = get_lyrics_state();
            app.manage(lyrics_state);

            let ext_state = get_extension_state(app.app_handle().clone())?;
            app.manage(ext_state);

            initial(app.state());

            app.listen("deep-link://new-url", |url| {
                println!("got url {:?}", url);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

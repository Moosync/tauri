use std::thread;

use database::database::Database;
use file_scanner::scanner::ScannerHolder;
use macros::generate_command;
use preferences::preferences::PreferenceConfig;
use serde_json::Value;
use tauri::{async_runtime, App, AppHandle, Emitter, Manager, State};
use types::errors::Result;

use crate::{providers::handler::ProviderHandler, scanner::start_scan};

const UI_KEYS: &[&str] = &[
    "prefs.system_settings",
    "prefs.queue_settings",
    "prefs.audio_settings",
    "prefs.gapless_skip",
    "prefs.volume_persist_mode",
    "prefs.spotify.enable",
    "prefs.spotify.username",
    "prefs.spotify.password",
    "prefs.themes.active_theme",
];

macro_rules! generate_states {
    ($app:expr, $( $state_type:ty ),*) => {
        {
            // Create a tuple to hold the state variables
            let tuple = ( $( $app.state::<$state_type>().clone() ),* );
            tuple
        }
    };
}

#[tracing::instrument(level = "trace", skip(app))]
pub fn handle_pref_changes(app: AppHandle) {
    async_runtime::spawn(async move {
        let pref_config: State<PreferenceConfig> = app.state::<PreferenceConfig>().clone();
        let receiver = pref_config.get_receiver();
        for (key, value) in receiver {
            tracing::info!("Received key: {} value: {}", key, value);
            if UI_KEYS.contains(&key.as_str()) {
                let app = app.clone();
                tracing::info!("Emitting preference-changed event");
                if let Err(e) = app.emit("preference-changed", (key.clone(), value.clone())) {
                    tracing::error!("Error emitting preference-changed event{}", e);
                } else {
                    tracing::info!("Emitted preference-changed event");
                }
            }

            if key == "prefs.music_paths" || key == "prefs.exclude_music_paths" {
                let app = app.clone();
                thread::spawn(move || {
                    let app = app.clone();
                    let (pref_config, scanner, database) =
                        generate_states!(app, PreferenceConfig, ScannerHolder, Database);
                    if let Err(e) = start_scan(scanner, database, pref_config, None, true) {
                        tracing::error!("{}", e);
                    }
                });
            }

            if key.starts_with("prefs.youtube") {
                let app = app.clone();
                async_runtime::spawn(async move {
                    let app = app.clone();
                    let provider_state: State<ProviderHandler> = app.state();
                    provider_state.initialize_provider("youtube".into()).await;
                });
            }

            if key.starts_with("prefs.spotify") {
                let app = app.clone();
                async_runtime::spawn(async move {
                    let app = app.clone();
                    let provider_state: State<ProviderHandler> = app.state();
                    provider_state.initialize_provider("spotify".into()).await;
                });
            }
        }
    });
}

#[tracing::instrument(level = "trace", skip(app))]
pub fn get_preference_state(app: &mut App) -> Result<PreferenceConfig> {
    let data_dir = app.path().app_config_dir()?;
    PreferenceConfig::new(data_dir)
}

#[tracing::instrument(level = "trace", skip(app))]
pub fn initial(app: &mut App) {
    let pref_config: State<PreferenceConfig> = app.state();
    if !pref_config.has_key("thumbnail_path") {
        let path = app.path().app_local_data_dir().unwrap().join("thumbnails");
        let _ = pref_config.save_selective("thumbnail_path".to_string(), Some(path));
    }

    if !pref_config.has_key("artwork_path") {
        let path = app.path().app_local_data_dir().unwrap().join("artwork");
        let _ = pref_config.save_selective("artwork_path".to_string(), Some(path));
    }
}

generate_command!(load_selective, PreferenceConfig, Value, key: String);
generate_command!(save_selective, PreferenceConfig, (), key: String, value: Option<Value>);
generate_command!(get_secure, PreferenceConfig, Value, key: String);
generate_command!(set_secure, PreferenceConfig, (), key: String, value: Option<Value>);
generate_command!(load_selective_array, PreferenceConfig, Value, key: String);

use database::cache::CacheHolder;
use librespot::{
    spirc::ParsedToken, utils::event_to_map, Bitrate, Cache, ConnectConfig, Credentials,
    DeviceType, LibrespotHolder, PlayerConfig, REGISTERED_EVENTS,
};
use macros::{generate_command, generate_command_cached};

use preferences::preferences::PreferenceConfig;
use tauri::{AppHandle, Emitter, Manager, State, Window};
use types::{canvaz::CanvazResponse, errors::Result};

#[tracing::instrument(level = "trace", skip())]
pub fn get_librespot_state() -> LibrespotHolder {
    LibrespotHolder::new()
}

#[tracing::instrument(level = "trace", skip(app, window, librespot))]
#[tauri::command()]
pub fn initialize_librespot(
    app: AppHandle,
    window: Window,
    librespot: State<LibrespotHolder>,
) -> Result<()> {
    let prefs: State<PreferenceConfig> = app.state();
    let username: String = prefs.load_selective("spotify.username".into())?;
    let password: String = prefs.load_selective("spotify.password".into())?;

    tracing::info!(
        "Initializing librespot {}@{}",
        username.trim(),
        password.trim()
    );

    let credentials = Credentials::with_password(username, password);

    let player_config = PlayerConfig {
        bitrate: Bitrate::Bitrate320,
        ..Default::default()
    };

    let connect_config = ConnectConfig {
        name: "Moosync".into(),
        device_type: DeviceType::Computer,
        initial_volume: Some(0),
        has_volume_ctrl: true,
        is_group: false,
    };

    let credentials_path = app.path().app_config_dir()?;
    let audio_path = app.path().app_cache_dir()?;
    let cache_config = Cache::new(
        Some(credentials_path.clone()),
        Some(credentials_path),
        Some(audio_path),
        None,
    )?;

    librespot.initialize(
        credentials,
        player_config,
        connect_config,
        cache_config,
        "".to_string(),
        "".to_string(),
    )?;

    // TODO: Check if event loop ends on closing librespot
    let events_channel = librespot.get_events_channel()?;
    tauri::async_runtime::spawn(async move {
        let events_channel = events_channel.lock().unwrap();
        loop {
            let event = events_channel.recv();
            match event {
                Ok(event) => {
                    let parsed_event = event_to_map(event.clone());

                    let registered_events = REGISTERED_EVENTS.lock().unwrap();
                    if registered_events.contains(&format!(
                        "librespot_event_{}",
                        parsed_event.get("event").unwrap(),
                    )) {
                        window
                            .emit(
                                format!("librespot_event_{}", parsed_event.get("event").unwrap(),)
                                    .as_str(),
                                parsed_event,
                            )
                            .unwrap();
                    }
                }
                Err(e) => {
                    tracing::error!("Ending event loop {:?}", e);
                    break;
                }
            }
        }
    });

    Ok(())
}

generate_command!(librespot_play, LibrespotHolder, (),);
generate_command!(librespot_pause, LibrespotHolder, (),);
generate_command!(librespot_close, LibrespotHolder, (),);
generate_command!(librespot_load, LibrespotHolder, (), uri: String, autoplay: bool);
generate_command!(librespot_seek, LibrespotHolder, (), pos: u32);
generate_command!(librespot_volume, LibrespotHolder, (), volume: u16);
generate_command!(librespot_get_token, LibrespotHolder, ParsedToken, scopes: String);
generate_command!(register_event, LibrespotHolder, (), event: String);
generate_command_cached!(get_canvaz, LibrespotHolder, CanvazResponse, uri: String);

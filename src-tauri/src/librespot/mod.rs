use std::str::FromStr;

use database::cache::CacheHolder;
use librespot::{
    spirc::ParsedToken, utils::event_to_map, Bitrate, Cache, ConnectConfig, Credentials,
    DeviceType, LibrespotHolder, PlayerConfig, REGISTERED_EVENTS,
};
use macros::{generate_command, generate_command_cached};

use serde_json::Value;
use tauri::{AppHandle, Manager, State, Window};
use types::{
    canvaz::CanvazResponse,
    errors::errors::{MoosyncError, Result},
};

pub fn get_librespot_state() -> LibrespotHolder {
    LibrespotHolder::new()
}

#[tauri::command()]
pub fn initialize_librespot(
    app: AppHandle,
    window: Window,
    librespot: State<LibrespotHolder>,
    config: Value,
    id: String,
) -> Result<()> {
    println!(
        "Initializing librespot {:?}",
        serde_json::to_string_pretty(&config)
    );

    let auth_config = config.get("auth").unwrap();
    let connect_config = config.get("connectConfig").unwrap();

    let credentials = Credentials::with_password(
        auth_config
            .get("username")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
        auth_config
            .get("password")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
    );

    let player_config = PlayerConfig {
        bitrate: Bitrate::Bitrate320,
        ..Default::default()
    };

    let connect_config = ConnectConfig {
        name: connect_config
            .get("name")
            .unwrap()
            .as_str()
            .unwrap_or_default()
            .to_string(),
        device_type: DeviceType::from_str(
            connect_config
                .get("deviceType")
                .unwrap()
                .as_str()
                .unwrap_or_default(),
        )
        .map_err(|_| MoosyncError::String("Failed to parse device type".to_string()))?,
        initial_volume: connect_config
            .get("initialVolume")
            .unwrap()
            .as_u64()
            .map(|v| v as u16),
        has_volume_ctrl: connect_config
            .get("hasVolumeControl")
            .unwrap()
            .as_bool()
            .unwrap_or_default(),
    };

    let volume_ctrl = config
        .get("volumeCtrl")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string();

    let backend = "".to_string();

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
        backend,
        volume_ctrl,
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
                        "librespot_event_{}_{}",
                        parsed_event.get("event").unwrap(),
                        id.clone()
                    )) {
                        window
                            .emit(
                                format!(
                                    "librespot_event_{}_{}",
                                    parsed_event.get("event").unwrap(),
                                    id.clone()
                                )
                                .as_str(),
                                parsed_event,
                            )
                            .unwrap();
                    }
                }
                Err(e) => {
                    println!("Ending event loop {:?}", e);
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

use std::{collections::HashMap, sync::Mutex};

use macros::generate_command;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;


use types::errors::errors::Result;

pub struct OAuthHandler {
    pub oauth_map: Mutex<HashMap<String, String>>,
}

impl OAuthHandler {
    pub fn new() -> OAuthHandler {
        OAuthHandler {
            oauth_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_oauth_path(&self, path: String) -> Result<String> {
        let mut oauth_map = self.oauth_map.lock().unwrap();
        if let Some(channel) = oauth_map.get(path.as_str()) {
            return Ok(channel.to_string());
        }
        let id = Uuid::new_v4().to_string();
        oauth_map.insert(path, id.clone());
        Ok(id)
    }

    pub fn unregister_oauth_path(&self, path: String) -> Result<()> {
        let mut oauth_map = self.oauth_map.lock().unwrap();
        oauth_map.remove(&path);
        Ok(())
    }

    pub fn handle_oauth(&self, app: AppHandle, url: String) -> Result<()> {
        let oauth_map = self.oauth_map.lock().unwrap();
        let query = url.replace("moosync://", "");
        let path = query.split('?').nth(0).unwrap();
        if let Some(channel) = oauth_map.get(path) {
            app.emit(channel.as_str(), url)?;
        }

        Ok(())
    }
}

generate_command!(register_oauth_path, OAuthHandler, String, path: String);
generate_command!(unregister_oauth_path, OAuthHandler, (), path: String);
generate_command!(handle_oauth, OAuthHandler, (), app: AppHandle, url: String);

pub fn get_oauth_state() -> Result<OAuthHandler> {
    Ok(OAuthHandler::new())
}

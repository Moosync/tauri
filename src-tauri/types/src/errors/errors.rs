use std::{
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

#[cfg(feature = "core")]
use std::io;
#[cfg(feature = "core")]
use std::time::SystemTimeError;

#[cfg(feature = "core")]
use fast_image_resize::ResizeError;

#[cfg(feature = "core")]
use google_youtube3::Error as YoutubeError;

#[cfg(feature = "core")]
use jsonschema::ValidationError;
#[cfg(feature = "core")]
use rspotify::ClientError;

use serde_json::Value;
#[cfg(feature = "ui")]
use wasm_bindgen::JsValue;

#[cfg(feature = "core")]
use fast_image_resize::ImageBufferError;
#[cfg(feature = "core")]
use hex::FromHexError;
#[cfg(feature = "core")]
use image::ImageError;
#[cfg(feature = "core")]
use keyring::Error as KeyringError;
#[cfg(feature = "core")]
use librespot::core::Error as LibrespotError;
#[cfg(feature = "core")]
use lofty::error::LoftyError;
#[cfg(feature = "core")]
use rusty_ytdl::VideoError;

#[derive(Debug, thiserror::Error)]
pub enum MoosyncError {
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    Tauri(#[from] tauri::Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    Diesel(#[from] diesel::result::Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    IO(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    Youtube(#[from] VideoError),
    #[error(transparent)]
    DotPaths(#[from] json_dotpath::Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    SystemTimeError(#[from] SystemTimeError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    ImageBufferError(#[from] ImageBufferError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    ImageError(#[from] ImageError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    DifferentTypesOfPixelsError(#[from] ResizeError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    LoftyError(#[from] LoftyError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    JWalkError(#[from] jwalk::Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    Librespot(#[from] LibrespotError),
    #[error(transparent)]
    UTF8(#[from] FromUtf8Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    Reqwest(#[from] reqwest::Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    ProtoBuf(#[from] protobuf::Error),
    #[error("{0}")]
    String(String),
    #[cfg_attr(feature = "core", error("Error in media controls: {0:?}"))]
    #[cfg(feature = "core")]
    MediaControlError(souvlaki::Error),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    ZipError(#[from] zip::result::ZipError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    FSExtraError(#[from] fs_extra::error::Error),
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    SpotifyError(#[from] ClientError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    YoutubeError(#[from] YoutubeError),
    #[cfg(feature = "core")]
    #[error("Transfer control to provider: {0}")]
    SwitchProviders(String),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    HexError(#[from] FromHexError),
    #[cfg_attr(feature = "core", error(transparent))]
    #[cfg(feature = "core")]
    KeyringError(#[from] KeyringError),
    #[cfg(feature = "core")]
    #[cfg_attr(feature = "core", error("JSON validation failed: {0}"))]
    JSONValidationError(String),
}

#[cfg(feature = "ui")]
impl From<serde_wasm_bindgen::Error> for MoosyncError {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::String(value.to_string())
    }
}

#[cfg(feature = "core")]
impl<'a> From<Box<dyn Iterator<Item = ValidationError<'a>> + Sync + Send + 'a>> for MoosyncError {
    fn from(value: Box<dyn Iterator<Item = ValidationError<'a>> + Sync + Send + 'a>) -> Self {
        let mut res = String::new();
        for error in value {
            res.push_str(error.to_string().as_str());
            res.push('\n');
        }
        Self::JSONValidationError(res)
    }
}

#[cfg(feature = "ui")]
impl From<JsValue> for MoosyncError {
    fn from(value: JsValue) -> Self {
        let parsed: Value = serde_wasm_bindgen::from_value(value).unwrap();
        Self::String(format!("{}", parsed))
    }
}

#[cfg(feature = "core")]
impl From<souvlaki::Error> for MoosyncError {
    fn from(value: souvlaki::Error) -> Self {
        Self::MediaControlError(value)
    }
}

impl From<&'static str> for MoosyncError {
    fn from(value: &'static str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for MoosyncError {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl serde::Serialize for MoosyncError {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, MoosyncError>;

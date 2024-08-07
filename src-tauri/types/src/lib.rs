pub mod cache;
#[cfg(feature = "core")]
pub mod cache_schema;
pub mod canvaz;
pub mod entities;
pub mod errors;
pub mod mpris;
pub mod preferences;
#[cfg(feature = "core")]
pub mod schema;
pub mod songs;
pub mod traits;

pub mod providers;
pub mod ui;

#[cfg(feature = "core")]
pub mod oauth;

pub mod themes;
pub mod window;

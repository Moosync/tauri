use chrono::{Duration, NaiveTime, Timelike};
use types::songs::Song;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    pub fn listen(event: &str, cb: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
    pub fn emit(event: &str, value: JsValue) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    fn convertFileSrc(path: &str, protocol: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__MOOSYNC__"])]
    async fn getBlobUrl(src: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "__MOOSYNC__"])]
    fn createYTPlayer(element: &str) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[macro_export]
macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::utils::common::log(&format_args!($($t)*).to_string()))
}

pub fn format_duration(secs: f64) -> String {
    if secs < 0.0 {
        return "Live".into();
    }

    let duration = Duration::seconds(secs as i64);
    let formatted_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap() + duration;
    if formatted_time.hour() == 0 {
        formatted_time.format("%M:%S").to_string()
    } else {
        formatted_time.format("%H:%M:%S").to_string()
    }
}

pub fn convert_file_src(path: String) -> String {
    if path.is_empty() {
        return path;
    }

    if !path.starts_with("http:") && !path.starts_with("https:") {
        let res = convertFileSrc(path.as_str(), "asset");
        return res.as_string().unwrap();
    }
    path
}

pub async fn get_blob_url(src: String) -> String {
    let res = getBlobUrl(src.as_str()).await;
    console_log!("Got blob url {}", res.as_string().unwrap());
    res.as_string().unwrap()
}

pub fn get_low_img(song: &Song) -> String {
    if let Some(cover) = &song.song.song_cover_path_low {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_low.clone())
    {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = &song.song.song_cover_path_high {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_high.clone())
    {
        return convert_file_src(cover.to_string());
    }

    String::new()
}

pub fn get_high_img(song: &Song) -> String {
    if let Some(cover) = &song.song.song_cover_path_high {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_high.clone())
    {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = &song.song.song_cover_path_low {
        return convert_file_src(cover.to_string());
    }

    if let Some(cover) = song
        .album
        .as_ref()
        .and_then(|a| a.album_coverpath_low.clone())
    {
        return convert_file_src(cover.to_string());
    }

    String::new()
}

macro_rules! fetch_infinite {
    ($provider_store:expr, $provider:expr, $fetch_content:ident, $update_signal:expr, $($arg:expr),*) => {
            use types::providers::generic::Pagination;
            let mut pagination = Pagination::new_limit(50, 0);
            loop {
                let res = $provider_store.$fetch_content($provider.clone(), $($arg,)* pagination).await;
                if res.is_err() {
                    break;
                }

                let (mut res, next_page) = res.unwrap();
                let len = res.len() as u32;

                if len == 0 {
                    break;
                }

                pagination = next_page;

                $update_signal.update(|signal| {
                    signal.append(&mut res);
                });
            }
    };
}

pub fn listen_event<F>(event: &str, cb: F) -> js_sys::Function
where
    F: Fn(JsValue) + 'static,
{
    let closure = Closure::wrap(Box::new(move |data: JsValue| {
        cb(data);
    }) as Box<dyn Fn(JsValue)>);
    let res = listen(event, closure.into_js_value());

    let event = event.to_string();
    let data = Box::new(move || {
        let event = event.clone();
        let unlisten = wasm_bindgen_futures::JsFuture::from(res.clone());
        spawn_local(async move {
            let unlisten = unlisten.await.unwrap();
            if unlisten.is_function() {
                let func = js_sys::Function::from(unlisten);
                console_log!("Cleaning up listener for {}", event.clone());
                func.call0(&JsValue::NULL).unwrap();
            }
        });
    }) as Box<dyn FnMut()>;

    let unlisten = Closure::wrap(data);

    js_sys::Function::from(unlisten.into_js_value())
}

pub(crate) use fetch_infinite;
use wasm_bindgen_futures::spawn_local;

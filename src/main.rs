mod app;
mod components;
mod icons;
mod modals;
mod pages;
mod players;
mod providers;
mod store;
mod utils;

use app::*;
use leptos::*;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| {
        view! { <App/> }
    })
}

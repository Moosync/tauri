use leptos::{component, create_rw_signal, view, IntoView, SignalGet, SignalSet};

use crate::icons::{play_hover_icon::PlayHoverIcon, song_default_icon::SongDefaultIcon};

#[component]
pub fn LowImg<T>(
    #[prop()] cover_img: String,
    #[prop(default = true)] show_play_button: bool,
    #[prop()] play_now: T,
) -> impl IntoView
where
    T: Fn() + 'static,
{
    let show_default_cover_img = create_rw_signal(false);
    view! {
        <div class="col-auto img-container h-100 d-flex justify-content-start">
            <div class="img-container justify-content-around ms-auto coverimg align-self-center">
                {move || {
                    if !show_default_cover_img.get() {
                        view! {
                            <img
                                class="fade-in-image"
                                src=cover_img.clone()
                                on:error=move |_| { show_default_cover_img.set(true) }
                            />
                        }
                            .into_view()
                    } else {
                        view! { <SongDefaultIcon /> }.into_view()
                    }
                }}
                {if show_play_button {
                    view! {
                        <div
                            class="play-button-song-list d-flex justify-content-center"
                            on:click=move |_| play_now()
                        >
                            <PlayHoverIcon />
                        </div>
                    }
                        .into_view()
                } else {
                    view! {}.into_view()
                }}

            </div>
        </div>
    }
}

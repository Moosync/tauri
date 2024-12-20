use std::rc::Rc;

use leptos::{
    component, create_action, create_rw_signal, event_target_value, expect_context, spawn_local,
    view, IntoView, SignalGet, SignalGetUntracked, SignalSet,
};
use serde::Serialize;

use crate::{
    modals::common::GenericModal, store::provider_store::ProviderStore, utils::common::invoke,
};

#[tracing::instrument(level = "trace", skip(key, name, account_id))]
#[component]
pub fn LoginModal(
    #[prop()] key: String,
    #[prop()] name: String,
    account_id: String,
) -> impl IntoView {
    let having_trouble = create_rw_signal(false);
    let code = create_rw_signal(String::new());
    let url = create_rw_signal(String::new());

    let provider_store = expect_context::<Rc<ProviderStore>>();

    let provider_store_cloned = provider_store.clone();
    let key_cloned = key.clone();

    let authorize = create_action(move |code: &String| {
        let provider_store = provider_store.clone();
        let code = code.clone();
        let key = key.clone();

        async move { provider_store.provider_authorize(key, code).await }
    });

    spawn_local(async move {
        let ret = provider_store_cloned
            .provider_login(key_cloned, account_id)
            .await
            .unwrap();
        url.set(ret);
    });

    let open_external = move |_| {
        let url = url.get();

        #[derive(Serialize)]
        struct OpenExternalArgs {
            url: String,
        }

        spawn_local(async move {
            let _ = invoke(
                "open_external",
                serde_wasm_bindgen::to_value(&OpenExternalArgs { url }).unwrap(),
            )
            .await;
        });
    };

    view! {
        <GenericModal size=move || "modal-sm".into()>
            <div class="w-100 h-100">
                <div class="container response-container">
                    <div class="row no-gutters d-flex">
                        <div class="col-auto title">Logging in to</div>
                        <div class="col-auto title ml-1" style="color: var(--accent)">
                            {name}
                        </div>
                    </div>

                    {move || {
                        if !having_trouble.get() {
                            view! {
                                <div>
                                    <div class="row">
                                        <div class="col mt-4 waiting">
                                            Waiting for response from your browser
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col d-flex justify-content-center">
                                            <div
                                                on:click=open_external
                                                class="start-button button-grow mt-4 d-flex justify-content-center align-items-center"
                                            >
                                                Open browser
                                            </div>
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div
                                            class="col not-working-text mt-3"
                                            on:click=move |_| having_trouble.set(true)
                                        >
                                            Having trouble?
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_view()
                        } else {
                            view! {
                                <div>
                                    <div class="row">
                                        <div class="col mt-4 waiting">
                                            Paste this link in your browser...
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col">
                                            <input
                                                class="ext-input mt-3"
                                                readonly
                                                prop:value=move || url.get()
                                            />
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col mt-4 waiting">
                                            Then enter the code shown after the login process has completed
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col">
                                            <input
                                                class="login-input mt-3"
                                                placeholder="Code"
                                                on:input=move |ev| code.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col d-flex justify-content-center">
                                            <div
                                                class="start-button button-grow mt-4 d-flex justify-content-center align-items-center"
                                                on:click=move |_| authorize.dispatch(code.get_untracked())
                                            >
                                                Submit
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_view()
                        }
                    }}

                </div>
            </div>
        </GenericModal>
    }
}

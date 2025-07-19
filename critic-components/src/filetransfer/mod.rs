//! A component used to transfer files to the server

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use web_sys::{File, MouseEvent};

mod components;
mod services;

use components::{
    dropzone::DropzonePreview,
    messages::{TransferComplete, TransferFailed},
};

#[component]
pub fn TransferPage(msname: String) -> impl IntoView {
    let files = RwSignal::new(Vec::<SendWrapper<File>>::new());

    let transfer_action = Action::new_local(move |files: &Vec<SendWrapper<File>>| {
        let selected_files = files.iter().map(|wrapped| wrapped.clone().take()).collect::<Vec<_>>();
        let name = msname.clone();
        async move {
            let res = services::transfer_file(&selected_files, name).await;
            if let Err(ref e) = res {
                leptos::logging::log!("{e}");
            };
            res
        }
    });
    let transfer_pending = transfer_action.pending();
    let transfer_reply = transfer_action.value();

    view! {
        <div class="flex items-center justify-center w-full p-2 md:p-8">
            <Show when=move || transfer_reply.get().is_none()>
            <DropzonePreview
                files=files
                transfer_pending=transfer_pending
                on_transfer=move |ev: MouseEvent| {
                    ev.prevent_default();
                    transfer_action.dispatch_local(files.get());
                } />
            </Show>

            <Show when=move || transfer_reply.get().is_some()>

                <Show when=move || transfer_reply.get().unwrap().is_ok()>
                <TransferComplete
                    on_continue=move |ev: MouseEvent| {
                        ev.prevent_default();
                        transfer_reply.set(None);
                    } />
                </Show>

                <Show when=move || transfer_reply.get().unwrap().is_err()>
                <TransferFailed
                    on_try_again=move |ev: MouseEvent| {
                        ev.prevent_default();
                        transfer_reply.set(None);
                    } />
                </Show>
            </Show>
        </div>
    }
}

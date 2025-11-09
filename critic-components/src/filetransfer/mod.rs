//! A component used to transfer files to the server
//!
//! Code taken in large parts from https://github.com/edinsonjim/file-uploader-example
//! The Code in this Module is NOT covered by this projects main license.

use leptos::prelude::*;
use send_wrapper::SendWrapper;
use web_sys::{File, MouseEvent};

mod components;
mod services;

use components::{
    dropzone::{DropzonePreview, DropzonePreviewSingle},
    messages::{TransferComplete, TransferFailed},
};

#[component]
pub fn TransferPage(msname: String) -> impl IntoView {
    let files = RwSignal::new(Vec::<SendWrapper<File>>::new());

    let transfer_action = Action::new_local(move |files: &Vec<SendWrapper<File>>| {
        let selected_files = files
            .iter()
            .map(|wrapped| wrapped.clone().take())
            .collect::<Vec<_>>();
        let name = msname.clone();
        async move { services::transfer_files(&selected_files, &name).await }
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
                    }
                />
            </Show>

            <Show when=move || transfer_reply.get().is_some()>

                <Show
                    when=move || transfer_reply.get().unwrap().err.iter().all(|x| x.is_none())
                    fallback=move || {
                        view! {
                            <TransferFailed
                                errs=transfer_reply.get().unwrap().err
                                filenames=files.read().iter().map(|f| f.name()).collect()
                                on_try_again=move |ev: MouseEvent| {
                                    ev.prevent_default();
                                    transfer_reply.set(None);
                                }
                            />
                        }
                    }
                >
                    <TransferComplete on_continue=move |ev: MouseEvent| {
                        ev.prevent_default();
                        transfer_reply.set(None);
                    } />
                </Show>
            </Show>
        </div>
    }
}

/// Transfer (upload) for a new model in the given category
#[component]
pub fn TransferModel(
    model_type: critic_shared::ModelType,
    on_new: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let file: RwSignal<Option<SendWrapper<web_sys::File>>> = RwSignal::new(None);

    let transfer_action = Action::new_local(move |file: &SendWrapper<File>| {
        let file_to_transfer = file.clone().take();
        async move {
            let res = services::transfer_model(file_to_transfer, model_type).await;
            on_new();
            res
        }
    });
    let transfer_pending = transfer_action.pending();
    let transfer_reply = transfer_action.value();

    view! {
        <div class="flex items-center justify-center w-full p-2 md:p-8">
            <Show when=move || transfer_reply.get().is_none()>
                <DropzonePreviewSingle
                    file=file
                    transfer_pending=transfer_pending
                    on_transfer=move |ev: MouseEvent| {
                        ev.prevent_default();
                        if let Some(chosen_file) = file.get() {
                            transfer_action.dispatch_local(chosen_file);
                        }
                    }
                />
            </Show>

            <Show when=move || transfer_reply.get().is_some()>

                <Show
                    when=move || transfer_reply.get().unwrap().err.iter().all(|x| x.is_none())
                    fallback=move || {
                        view! {
                            <TransferFailed
                                errs=transfer_reply.get().unwrap().err
                                filenames=file.read().iter().map(|f| f.name()).collect()
                                on_try_again=move |ev: MouseEvent| {
                                    ev.prevent_default();
                                    transfer_reply.set(None);
                                }
                            />
                        }
                    }
                >
                    <TransferComplete on_continue=move |ev: MouseEvent| {
                        ev.prevent_default();
                        transfer_reply.set(None);
                    } />
                </Show>
            </Show>
        </div>
    }
}

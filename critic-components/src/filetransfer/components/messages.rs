use leptos::prelude::*;
use web_sys::MouseEvent;

use crate::{
    filetransfer::components::buttons::Button,
    icons::{CheckIcon, InfoIcon},
};

#[component]
pub fn TransferComplete(on_continue: impl Fn(MouseEvent) + 'static) -> impl IntoView {
    let (busy_reader, _) = signal(false);

    view! {
        <div class="w-full max-w-lg p-3 bg-white border border-gray-200 rounded-lg md:p-6 sm:p-2">
            <div class="flex flex-col items-center gap-y-8">
                <CheckIcon inner_class="text-green-500 h-24 w-24" />

                <div class="flex flex-col items-center">
                    <div>Upload complete</div>
                    <div>Your files have been uploaded successfully.</div>
                </div>

                <Button
                    busy_reader=busy_reader
                    on_click=on_continue
                    label="Continue" />
            </div>
        </div>
    }
}

#[component]
pub fn TransferFailed(errs: Vec<Option<String>>, filenames: Vec<String>, on_try_again: impl Fn(MouseEvent) + 'static) -> impl IntoView {
    let (busy_reader, _) = signal(false);

    view! {
        <div class="w-full max-w-lg p-3 bg-white border border-gray-200 rounded-lg md:p-6 sm:p-2">
            <div class="flex flex-col items-center gap-y-8">
                <InfoIcon inner_class="h-24 w-24 text-rose-500" />

                <div class="flex flex-col items-center">
                    <div>Upload failed</div>
                    <div>Sorry! Something went wrong.</div>
                    {
                        if errs.len() != filenames.len() {
                            leptos::logging::log!("Errors and filenames are not the same length in TransferFailed.");
                            None
                        } else {
                            Some(errs.into_iter().enumerate().map(|(idx, e)| e.map(|msg| format!("File {}: {msg}", filenames.get(idx).unwrap()))).collect_view())
                        }
                    }
                </div>

                <Button
                    busy_reader=busy_reader
                    on_click=on_try_again
                    label="Try again" />
            </div>
        </div>
    }
}

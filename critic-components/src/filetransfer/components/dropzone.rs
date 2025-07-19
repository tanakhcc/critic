use leptos::html::Label;
use leptos::prelude::*;
use leptos_use::{use_drop_zone_with_options, UseDropZoneOptions, UseDropZoneReturn};
use send_wrapper::SendWrapper;
use web_sys::wasm_bindgen::JsCast;
use web_sys::{js_sys, Event, File, HtmlInputElement, MouseEvent};

use crate::filetransfer::components::{buttons::Button, file::FileList};

#[component]
pub fn DropzoneBar() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center pt-5 pb-6">
            <svg class="w-8 h-8 mb-4 text-violet-500" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 16">
                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 13h3a3 3 0 0 0 0-6h-.025A5.56 5.56 0 0 0 16 6.5 5.5 5.5 0 0 0 5.207 5.021C5.137 5.017 5.071 5 5 5a4 4 0 0 0 0 8h2.167M10 15V6m0 0L8 8m2-2 2 2"/>
            </svg>
            <p class="mb-2 text-sm text-violet-500">
                <span class="font-semibold">
                    Click to upload
                </span> or drag and drop
            </p>
            <p class="text-xs text-violet-500">
                Up to 50GiB
            </p>
        </div>
    }
}

#[component]
pub fn DropzonePreview(
    files: RwSignal<Vec<SendWrapper<File>>>,
    transfer_pending: Memo<bool>,
    on_transfer: impl Fn(MouseEvent) -> () + 'static + Send + Sync + Clone,
) -> impl IntoView {
    let (dropped, set_dropped) = signal(false);

    let drop_zone_el = NodeRef::<Label>::new();

    let UseDropZoneReturn {
        is_over_drop_zone: _,
        files: _,
    } = use_drop_zone_with_options(
        drop_zone_el,
        UseDropZoneOptions::default()
            .on_drop(move |ev| {
                files.update(move |f| *f = ev.files.into_iter().map(SendWrapper::new).collect());
                set_dropped.set(true);
            })
            .on_enter(move |_| set_dropped.set(false)),
    );

    let on_change_file = move |ev: Event| {
        ev.stop_propagation();

        let input_file_el = ev
            .target()
            .unwrap()
            .dyn_ref::<HtmlInputElement>()
            .unwrap()
            .clone();

        let selected_files: Vec<SendWrapper<File>> = input_file_el
            .files()
            .map(|f| js_sys::Array::from(&f).to_vec())
            .unwrap_or_default()
            .into_iter()
            .map(web_sys::File::from)
            .map(SendWrapper::new)
            .collect();

        files.update(move |f| *f = selected_files);
        set_dropped.set(true);
    };

    view! {
        <div class="w-full max-w-lg p-3 bg-white border border-gray-200 rounded-lg md:p-6 sm:p-2">
            <div class="drop_zone_file_container">
                <label node_ref=drop_zone_el
                    for="drop_zone_input"
                    class="flex flex-col items-center justify-center w-full h-28 border-2 border-violet-300 border-dashed rounded-lg cursor-pointer bg-violet-50 hover:bg-violet-100">
                    <DropzoneBar />
                </label>

                <input id="drop_zone_input"
                    class="hidden"
                    type="file"
                    multiple
                    on:change=on_change_file />

                <Show when=move ||dropped.get()>
                <div class="flow-root mt-3">
                    <FileList
                        files=files
                        transfer_pending=transfer_pending
                        dropped_setter=set_dropped
                    />
                </div>
                </Show>


                <Show when=move ||dropped.get()>
                <div class="mt-3">
                    <Button
                        label="Transfer"
                        busy_label="Transferring..."
                        busy_reader=transfer_pending
                        on_click=on_transfer.clone()
                        />
                </div>
                </Show>
            </div>
        </div>
    }
}

use human_bytes::human_bytes;
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use web_sys::File;

use crate::{filetransfer::components::buttons::ButtonIcon, icons::TrashIcon};

#[component]
pub fn FileItem(
    name: String,
    size: f64,
    #[prop(into)] processing_reader: Signal<bool>,
    on_remove: impl Fn(MouseEvent) -> () + 'static,
) -> impl IntoView {
    view! {
        <li class="border border-gray-200 rounded-lg mb-2 p-3">
            <div class="flex items-center">
                <div class="flex-1 min-w-0">
                    <p class="text-sm font-medium text-gray-900 truncate">
                        {name}
                    </p>
                    <p class="text-sm text-gray-500 truncate">
                        Size: {human_bytes(size)}
                    </p>
                </div>
                <ButtonIcon
                    busy_reader=processing_reader
                    on_click=on_remove
                    inner_icon=||view! { <TrashIcon inner_class="w-6 h-6" /> }
                    >
                </ButtonIcon>
            </div>
        </li>
    }
}

#[component]
pub fn FileList(
    /// [`send_wrapper`] is only compiled in hydrate, so this will never run in a multi-threaded
    /// environment. Accessing SendWrapper<File> is guaranteed to be safe.
    files: RwSignal<Vec<send_wrapper::SendWrapper<File>>>,
    transfer_pending: Memo<bool>,
    dropped_setter: WriteSignal<bool>,
) -> impl IntoView {
    view! {
        <ul role="list">
            <For
                each=move || files.get()
                key=|f| f.name()
                let:file>
            <FileItem
                name=file.name()
                size=file.size()
                processing_reader=transfer_pending
                on_remove=move |_| {
                    files.update(|files| {
                        let index = files.iter().position(|f_iter| f_iter.name().eq(&file.name())).unwrap();
                        files.remove(index);
                    });

                    if files.get().len() == 0 {
                        dropped_setter.set(false);
                    }
                }
            />
            </For>
        </ul>
    }
}

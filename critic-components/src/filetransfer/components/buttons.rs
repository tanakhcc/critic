use crate::icons::SpinIcon;
use leptos::prelude::*;
use web_sys::MouseEvent;

#[component]
pub fn ButtonIcon<F, IV>(
    #[prop(into)] busy_reader: Signal<bool>,
    on_click: impl Fn(MouseEvent) + 'static,
    inner_icon: F,
) -> impl IntoView
where
    F: Fn() -> IV,
    IV: IntoView,
{
    view! {
        <button
            disabled=move || busy_reader.get()
            class="inline-flex items-center text-base font-semibold text-gray-500 cursor-pointer ml-2 p-2 rounded-full hover:text-slate-500 hover:bg-slate-100"
            on:click=move |ev| {
                if !busy_reader.get() {
                    on_click(ev);
                }
            }
        >
            {inner_icon()}
        </button>
    }
}

#[component]
pub fn Button(
    #[prop(into)] busy_reader: Signal<bool>,
    on_click: impl Fn(MouseEvent) + 'static,
    #[prop(default = "")] label: &'static str,
    #[prop(default = "")] busy_label: &'static str,
) -> impl IntoView {
    view! {
        <button
            class="h-9 flex justify-center items-center space-x-4 w-full text-slate-50 bg-slate-700 hover:bg-slate-800 focus:ring-4 focus:outline-none focus:ring-slate-300 font-medium rounded-lg text-sm text-center"
            disabled=move || busy_reader.get()
            on:click=move |ev| {
                if !busy_reader.get() {
                    on_click(ev);
                }
            }
        >

            <Show when=move || busy_reader.get()>
                <SpinIcon inner_class="animate-spin h-5 w-5 mr-2 text-slate-50" />
            </Show>

            {move || if busy_reader.get() { busy_label.to_string() } else { label.to_string() }}
        </button>
    }
}

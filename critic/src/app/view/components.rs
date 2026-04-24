//! Subcomponents for the View Page

use critic_format::page_to_xml;
use critic_shared::BaselineStoreFields;
use leptos::prelude::*;

use super::KeyedBaseline;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum SelectedTool {
    Select,
    NewLine,
    EditLine,
}

const BASE_CLASSES: &str = "m-2 size-10 rounded-sm border-slate-600 p-1";

#[component]
pub(super) fn toolbar(
    tool: RwSignal<SelectedTool>,
    on_save: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let select_selected = move || tool.get() == SelectedTool::Select;
    let new_selected = move || tool.get() == SelectedTool::NewLine;
    let edit_selected = move || tool.get() == SelectedTool::EditLine;

    view! {
        <div class="grid grid-cols-4">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class=BASE_CLASSES
                class=(["border-2", "bg-slate-800"], move || select_selected())
                class=(["hover:stroke-sky-300"], move || !select_selected())
                on:click=move |_evt| {
                    tool.set(SelectedTool::Select);
                }
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M15.042 21.672 13.684 16.6m0 0-2.51 2.225.569-9.47 5.227 7.917-3.286-.672ZM12 2.25V4.5m5.834.166-1.591 1.591M20.25 10.5H18M7.757 14.743l-1.59 1.59M6 10.5H3.75m4.007-4.243-1.59-1.59"
                />
            </svg>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class=BASE_CLASSES
                class=(["border-2", "bg-slate-800"], move || new_selected())
                class=(["hover:stroke-sky-300"], move || !new_selected())
                on:click=move |_evt| {
                    tool.set(SelectedTool::NewLine);
                }
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M12 9v6m3-3H9m12 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
                />
            </svg>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class=BASE_CLASSES
                class=(["border-2", "bg-slate-800"], move || edit_selected())
                class=(["hover:stroke-sky-300"], move || !edit_selected())
                on:click=move |_evt| {
                    tool.set(SelectedTool::EditLine);
                }
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="m16.862 4.487 1.687-1.688a1.875 1.875 0 1 1 2.652 2.652L10.582 16.07a4.5 4.5 0 0 1-1.897 1.13L6 18l.8-2.685a4.5 4.5 0 0 1 1.13-1.897l8.932-8.931Zm0 0L19.5 7.125M18 14v4.75A2.25 2.25 0 0 1 15.75 21H5.25A2.25 2.25 0 0 1 3 18.75V8.25A2.25 2.25 0 0 1 5.25 6H10"
                />
            </svg>
            <svg
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
                stroke-width="1.5"
                stroke="currentColor"
                class="m-2 size-10 rounded-sm bg-slate-600 p-1 hover:stroke-sky-300"
                on:click=move |_evt| { on_save() }
            >
                <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M10.125 2.25h-4.5c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125v-9M10.125 2.25h.375a9 9 0 0 1 9 9v.375M10.125 2.25A3.375 3.375 0 0 1 13.5 5.625v1.5c0 .621.504 1.125 1.125 1.125h1.5a3.375 3.375 0 0 1 3.375 3.375M9 15l2.25 2.25L15 12"
                />
            </svg>
        </div>
    }
}

#[component]
pub(super) fn layers() -> impl IntoView {
    view! {
        <div class="border-t-2 border-slate-600">
            <h1 class="text-center text-xl">Aleppo XYZ</h1>
            <ul class="mx-2">
                <li class="flex flex-row">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M3.75 6.75h16.5M3.75 12H12m-8.25 5.25h16.5"
                        />
                    </svg>
                    <p class="mx-1">Segmentation</p>
                </li>
                <li class="flex flex-row">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z"
                        />
                    </svg>

                    <p class="mx-1">OCR results</p>
                </li>
                <li class="flex flex-row">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M17.982 18.725A7.488 7.488 0 0 0 12 15.75a7.488 7.488 0 0 0-5.982 2.975m11.963 0a9 9 0 1 0-11.963 0m11.963 0A8.966 8.966 0 0 1 12 21a8.966 8.966 0 0 1-5.982-2.275M15 9.75a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                        />
                    </svg>
                    <p class="mx-1">Transcriptions</p>
                </li>
                <li class="flex flex-row">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M9 12.75 11.25 15 15 9.75M21 12c0 1.268-.63 2.39-1.593 3.068a3.745 3.745 0 0 1-1.043 3.296 3.745 3.745 0 0 1-3.296 1.043A3.745 3.745 0 0 1 12 21c-1.268 0-2.39-.63-3.068-1.593a3.746 3.746 0 0 1-3.296-1.043 3.745 3.745 0 0 1-1.043-3.296A3.745 3.745 0 0 1 3 12c0-1.268.63-2.39 1.593-3.068a3.745 3.745 0 0 1 1.043-3.296 3.746 3.746 0 0 1 3.296-1.043A3.746 3.746 0 0 1 12 3c1.268 0 2.39.63 3.068 1.593a3.746 3.746 0 0 1 3.296 1.043 3.746 3.746 0 0 1 1.043 3.296A3.745 3.745 0 0 1 21 12Z"
                        />
                    </svg>

                    <p class="mx-1">Reconciliations</p>
                </li>
            </ul>
        </div>
    }
}

/// Show Information on a Baseline
#[component]
pub(super) fn Information(selected: ReadSignal<Option<KeyedBaseline>>) -> impl IntoView {
    {
        move || {
            selected.read().map(
                |sel| match page_to_xml(sel.content().get(), "".to_string()) {
                    Ok(xml) => leptos::either::Either::Left(view! {
                        <p>This baseline has the following XML:</p>
                        <p>{xml}</p>
                    }),
                    Err(e) => leptos::either::Either::Right(view! {
                        <p>Problem while searlizing the Data for this line into XML:</p>
                        <p>{e.to_string()}</p>
                    }),
                },
            )
        }
    }
}

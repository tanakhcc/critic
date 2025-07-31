//! Components and server functions to show transcripitions that are todo

use critic_shared::PageTodo;
use leptos::{ev::keydown, prelude::*};
use leptos_router::hooks::query_signal;
use leptos_use::use_event_listener;

use crate::app::TopLevelPosition;

#[server]
pub async fn get_pages_by_query<'a>(query: String) -> Result<Vec<PageTodo>, ServerFnError> {
    let config: std::sync::Arc<critic_server::config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    let username = "TODO-THE-User-name";
    let res = critic_server::db::get_pages_by_query(&config.db, &query, username).await;
    match res {
        Ok(x) => Ok(x),
        Err(e) => {
            tracing::warn!("Failed loading page list: {e}");
            Err(ServerFnError::new(e.to_string()))
        }
    }
}

#[component]
pub fn TranscribeTodoList() -> impl IntoView {
    let set_top_level_pos =
        use_context::<WriteSignal<TopLevelPosition>>().expect("App provides TopLevelPosition");
    *set_top_level_pos.write() = TopLevelPosition::Transcribe;

    let (query, set_query) = query_signal::<String>("psq");

    // the keyboard-shortcut listener
    let search_node_ref = NodeRef::<leptos::html::Input>::new();
    let _cleanup = use_event_listener(search_node_ref, keydown, move |evt| {
        if evt.key_code() == 13 {
            let x = search_node_ref.get().expect("statically mounted").value();
            set_query.set(if x.is_empty() { None } else { Some(x) });
        }
    });

    let pages = Resource::new(
        move || query.get(),
        async |new_query| {
            if let Some(qstr) = new_query {
                get_pages_by_query(qstr).await
            } else {
                get_pages_by_query("".to_string()).await
            }
        },
    );

    // get pages from db based on search parameter
    // show list of pages that the server returned
    view! {
        <div class="flex h-full flex-col">
            <div class="flex flex-row justify-center">
                <h1 class="text-6xl font-semibold p-10">Start a new Transcription for ...</h1>
            </div>
            <div class="flex flex-row justify-center mb-2">
                <div class="flex w-2/5 flex-row justify-start rounded-4xl border-2 border-slate-600 bg-slate-800 p-4 text-xl shadow-sky-600 shadow-md">
                    <label for="page-search">
                        <svg
                            xmlns="http://www.w3.org/2000/svg"
                            fill="none"
                            viewBox="0 0 24 24"
                            stroke-width="1.5"
                            stroke="currentColor"
                            class="size-6 text-slate-300"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="m21 21-5.197-5.197m0 0A7.5 7.5 0 1 0 5.196 5.196a7.5 7.5 0 0 0 10.607 10.607Z"
                            />
                        </svg>
                    </label>
                    <input
                        node_ref=search_node_ref
                        id="page-search"
                        class="w-0 grow border-0 font-mono text-slate-400"
                        type="search"
                    />
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6 text-slate-300"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="m7.49 12-3.75 3.75m0 0 3.75 3.75m-3.75-3.75h16.5V4.499"
                        />
                    </svg>
                </div>
            </div>
            <div class="mt-8 flex min-h-24 grow flex-row justify-center overflow-y-auto mb-10 no-scrollbar">
                <div id="page-listing" class="text-md table w-4/5">
                    <div class="table-row-group">
                        <a
                            href="/transcribe/:msname/:pagename"
                            class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl"
                        >
                            <div class="table-cell border-r border-inherit p-2">ML115</div>
                            <div class="table-cell border-r border-inherit p-2">Page 014</div>
                            <div class="table-cell border-r border-inherit p-2">
                                Ps 32 : 4 - Ps 34 : 7
                            </div>
                            <div class="table-cell border-r border-inherit p-2">
                                <div>
                                    <span class="font-extrabold">2</span>
                                    " started/ "
                                    <span class="font-extrabold">1</span>
                                    " published"
                                </div>
                                <div>You have started this page.</div>
                            </div>
                        </a>
                    </div>
                </div>
            </div>
        </div>
    }
}

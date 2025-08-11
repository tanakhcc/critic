//! Components and server functions to show transcripitions that are todo

use critic_shared::{OwnStatus, PageTodo, PublishedTranscriptions};
use leptos::{either::Either, ev::keydown, prelude::*};
use leptos_router::hooks::query_signal;
use leptos_use::use_event_listener;

use crate::app::TopLevelPosition;

#[server]
pub async fn get_pages_by_query(
    query: String,
    page: Option<i32>,
) -> Result<Vec<PageTodo>, ServerFnError> {
    use critic_server::auth::AuthSession;
    use leptos_axum::extract;
    let config: std::sync::Arc<critic_server::config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;

    let auth_session = match extract::<AuthSession>().await {
        Ok(x) => x,
        Err(e) => {
            let msg = format!("Failed to get AuthSession: {e}");
            tracing::warn!(msg);
            return Err(ServerFnError::new(msg));
        }
    };
    let Some(user) = auth_session.user else {
        return Err(ServerFnError::new("No usersession available"));
    };

    let res = critic_server::db::get_pages_by_query(
        &config.db,
        &query,
        &user.username,
        page.unwrap_or_default(),
    )
    .await;
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
    let (page, set_page) = query_signal::<i32>("page");

    // Set Query when user presses enter while focused on the search input
    let search_node_ref = NodeRef::<leptos::html::Input>::new();
    let _cleanup = use_event_listener(search_node_ref, keydown, move |evt| {
        if evt.key_code() == 13 {
            let x = search_node_ref.get().expect("statically mounted").value();
            set_query.set(if x.is_empty() { None } else { Some(x) });
        }
    });

    let pages = Resource::new(
        move || (query.get(), page.get()),
        async |(new_query, new_page)| {
            get_pages_by_query(new_query.unwrap_or_default(), new_page).await
        },
    );
    let todos_rendered = move || {
        pages.get().map(|pages_res| pages_res.map(|pages_ok| pages_ok.into_iter().map(|page_todo| view! {
            <div class="table-row-group">
                <a
                    href=format!(
                        "/transcribe/{}/{}",
                        &page_todo.manuscript_name,
                        &page_todo.page_name,
                    )
                    class="table-row border-b border-slate-600 py-3 text-xl shadow-sky-600 last:border-b-0 odd:bg-slate-800 even:bg-slate-600 hover:bg-sky-900 hover:shadow-2xl"
                >
                    <div class="table-cell border-r border-inherit p-2">
                        {page_todo.manuscript_name.clone()}
                    </div>
                    <div class="table-cell border-r border-inherit p-2">
                        {page_todo.page_name.clone()}
                    </div>
                    {if let (Some(start), Some(end)) = (
                        page_todo.verse_start,
                        page_todo.verse_end,
                    ) {
                        Either::Left(
                            view! {
                                <div class="table-cell border-r border-inherit p-2">
                                    {start}" - "{end}
                                </div>
                            },
                        )
                    } else {
                        Either::Right(
                            view! { <div class="table-cell border-r border-inherit p-2"></div> },
                        )
                    }}
                    <div class="table-cell border-r border-inherit p-2">
                        {match page_todo.transcriptions_started {
                            0 => ().into_any(),
                            started_total => {
                                view! {
                                    <div>
                                        <span class="font-extrabold">{started_total}</span>
                                        " started"
                                        {match page_todo.transcriptions_published {
                                            PublishedTranscriptions::None => ().into_any(),
                                            PublishedTranscriptions::One => {
                                                view! {
                                                    ", "
                                                    <span class="font-extrabold">1</span>
                                                    " published"
                                                }
                                                    .into_any()
                                            }
                                            PublishedTranscriptions::Two => {
                                                view! {
                                                    ", "
                                                    <span class="font-extrabold">2</span>
                                                    " published"
                                                }
                                                    .into_any()
                                            }
                                            PublishedTranscriptions::More => {
                                                view! {
                                                    ", "
                                                    <span class="font-extrabold">">2"</span>
                                                    " published"
                                                }
                                                    .into_any()
                                            }
                                        }}
                                    </div>
                                    {match page_todo.this_user_status {
                                        OwnStatus::None => ().into_any(),
                                        OwnStatus::Started => {
                                            view! { <div>You have started this page.</div> }.into_any()
                                        }
                                        OwnStatus::Published => {
                                            view! { <div>You have published this page.</div> }
                                                .into_any()
                                        }
                                    }}
                                }
                                    .into_any()
                            }
                        }}
                    </div>
                </a>
            </div>
        }).collect_view()))
    };

    // get pages from db based on search parameter
    // show list of pages that the server returned
    view! {
        <div class="flex h-full flex-col relative">
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
                        placeholder="ms:<name> page:<nr> lang:<>"
                        type="search"
                        value=move || query.get()
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
                    <ErrorBoundary fallback=|errors| {
                        view! {
                            <div>
                                "Error: failed to get pages:"
                                <ul>
                                    {move || {
                                        errors
                                            .get()
                                            .into_iter()
                                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                            .collect::<Vec<_>>()
                                    }}
                                </ul>
                            </div>
                        }
                    }>
                        <Transition fallback=|| {
                            view! { <p>Loading Pages...</p> }
                        }>{move || todos_rendered()}</Transition>
                    </ErrorBoundary>
                </div>
            </div>
            // Buttons for pagination
            <div class="absolute bottom-2 left-4">
                <button
                    class="rounded-xl hover:bg-sky-600 p-2"
                    // hide if this is the first page (0-based)
                    class=("hidden", move || { page.read().unwrap_or_default() <= 0 })
                    // go back one page, if possible
                    on:click=move |_| {
                        set_page
                            .set({
                                if let Some(y) = page.get() {
                                    if y >= 1 { Some(y - 1) } else { None }
                                } else {
                                    None
                                }
                            })
                    }
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-10"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M15.75 19.5 8.25 12l7.5-7.5"
                        />
                    </svg>
                </button>
            </div>
            {move || {
                page.get()
                    .map(|x| {
                        view! {
                            <div class="absolute bottom-2 flex justify-center w-full">
                                <p>"Result Page "{x + 1}</p>
                            </div>
                        }
                    })
            }}
            <div class="absolute bottom-2 right-4">
                // we have no good way to know whether this is the las page, so we always allow the
                // button
                <button
                    class="rounded-xl hover:bg-sky-600 p-2"
                    on:click=move |_| set_page.set(Some(page.get().unwrap_or_default() + 1))
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-10"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="m8.25 4.5 7.5 7.5-7.5 7.5"
                        />
                    </svg>
                </button>
            </div>
        </div>
    }
}

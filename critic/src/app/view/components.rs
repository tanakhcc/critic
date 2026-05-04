//! Subcomponents for the View Page

use critic_components::editor::blocks::EditorBlock;
use critic_format::streamed::Block;
use critic_shared::{BaselineContentStoreFields, BaselineStoreFields};
use leptos::prelude::*;

use crate::app::view::line_editor::EditorWithTabs;

use super::KeyedBaseline;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub(super) enum SelectedTool {
    NewLine,
    EditLine,
    EditBoundary,
    Transcription,
    Reconciliation,
}

#[component]
fn Action(children: Children) -> impl IntoView {
    view! {
        <li class="rounded-sm border-2 p-1 border-black hover:text-sky-300 hover:bg-slate-800 hover:border-slate-600">
            <button class="flex flex-row">{children()}</button>
        </li>
    }
}

#[component]
fn Tool(
    children: Children,
    tool: RwSignal<SelectedTool>,
    this_tool: SelectedTool,
) -> impl IntoView {
    view! {
        <li
            class="rounded-sm border-2"
            class=(
                ["border-slate-600", "bg-slate-800", "underline"],
                move || tool.get() == this_tool,
            )
            class=(["border-black", "hover:text-sky-300"], move || tool.get() != this_tool)
        >
            <button
                class="flex flex-row"
                on:click=move |_evt| {
                    tool.set(this_tool);
                }
            >
                {children()}
            </button>
        </li>
    }
}

#[component]
pub(super) fn Sidebar(
    /// Name of the manuscript
    msname: String,
    /// Name of the page
    pagename: String,
    /// the currently selected tool
    tool: RwSignal<SelectedTool>,
    /// the regions in the segmentation
    regions: reactive_stores::Store<critic_shared::SegmentedPage>,
    /// should we autofocus when the user clicks a page?
    should_autofocus: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div id="sidebar" class="h-full w-52 bg-black">
            <div id="sidebar-header" class="mb-2 border-b-2 border-slate-600 p-2 pb-2">
                <h1 class="text-left text-xl">{msname}</h1>
                <h2 class="text-md text-left">{pagename}</h2>
            </div>
            <form class="px-2 border-b-2 border-slate-600">
                <label class="inline-flex items-center cursor-pointer">
                    <input
                        class="sr-only peer"
                        name="should_autofocus"
                        type="checkbox"
                        value="true"
                        prop:checked=should_autofocus
                        on:change:target=move |evt| {
                            should_autofocus.set(evt.target().checked());
                        }
                    />
                    <div class="relative w-9 h-5 bg-slate-600 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-brand-soft dark:peer-focus:ring-brand-soft rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-buffer after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-sky-600"></div>
                    <span class="select-none ms-3 text-sm font-medium text-heading">
                        Autofocus on Select
                    </span>
                </label>
            </form>
            <ul id="tools" class="mx-2">
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
                        ></path>
                    </svg>
                    <p class="mx-1">Segmentation</p>
                </li>
                <li>
                    <ul class="mx-4">
                        <Tool tool=tool this_tool=SelectedTool::NewLine>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke-width="1.5"
                                stroke="currentColor"
                                class="size-6"
                            >
                                <line x1="18" y1="6" x2="6" y2="18" />
                                <circle cx="19" cy="5" r="2"></circle>
                                <circle cx="5" cy="19" r="2"></circle>
                                <line x1="3" y1="7" x2="11" y2="7" />
                                <line x1="7" y1="3" x2="7" y2="11" />
                            </svg>
                            <p class="mx-1">New Line</p>
                        </Tool>
                        <Tool tool=tool this_tool=SelectedTool::EditLine>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke-width="1.5"
                                stroke="currentColor"
                                class="size-6"
                            >
                                <line x1="18" y1="6" x2="6" y2="18" />
                                <circle cx="19" cy="5" r="2"></circle>
                                <circle cx="5" cy="19" r="2"></circle>
                            </svg>
                            <p class="mx-1">Edit Line</p>
                        </Tool>
                        <Tool tool=tool this_tool=SelectedTool::EditBoundary>
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke-width="1.5"
                                stroke="currentColor"
                                class="size-6"
                            >
                                <polygon points="14,3 20,5 20,16 14,17 11,20 4,16 4,5" />
                            </svg>
                            <p class="mx-1">Edit Boundary</p>
                        </Tool>
                        <Action>
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
                                    d="M10.125 2.25h-4.5c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125v-9M10.125 2.25h.375a9 9 0 0 1 9 9v.375M10.125 2.25A3.375 3.375 0 0 1 13.5 5.625v1.5c0 .621.504 1.125 1.125 1.125h1.5a3.375 3.375 0 0 1 3.375 3.375M9 15l2.25 2.25L15 12"
                                />
                            </svg>
                            <p class="mx-1">Save</p>
                        </Action>
                        <Action>
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
                                    d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182m0-4.991v4.99"
                                />
                            </svg>
                            <p class="mx-1">Rerun OCR</p>
                        </Action>
                    </ul>
                </li>
                <Tool tool=tool this_tool=SelectedTool::Transcription>
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
                        ></path>
                    </svg>
                    <p class="mx-1">Transcription</p>
                </Tool>
                <li>
                    <ul class="mx-4">
                        <Action>
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
                                    d="M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178Z"
                                />
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                                />
                            </svg>

                            <p class="mx-1">Publish all</p>
                        </Action>
                    </ul>
                </li>
                <Tool tool=tool this_tool=SelectedTool::Reconciliation>
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
                        ></path>
                    </svg>
                    <p class="mx-1">Reconciliation</p>
                </Tool>
                <li>
                    <ul class="mx-4">
                        <Action>
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
                                    d="M10.125 2.25h-4.5c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125v-9M10.125 2.25h.375a9 9 0 0 1 9 9v.375M10.125 2.25A3.375 3.375 0 0 1 13.5 5.625v1.5c0 .621.504 1.125 1.125 1.125h1.5a3.375 3.375 0 0 1 3.375 3.375M9 15l2.25 2.25L15 12"
                                />
                            </svg>
                            <p class="mx-1">Save</p>
                        </Action>
                        <Action>
                            <svg
                                width="800px"
                                height="800px"
                                viewBox="-1 -1 17 17"
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                stroke="currentColor"
                                class="size-6"
                            >
                                <path
                                    fill-rule="evenodd"
                                    d="M10,0 L10,2.60002 C12.2108812,3.04881281 13.8920863,4.95644867 13.9950026,7.27443311 L14,7.5 L14,11.2676 C14.5978,11.6134 15,12.2597 15,13 C15,14.1046 14.1046,15 13,15 C11.8954,15 11,14.1046 11,13 C11,12.3166462 11.342703,11.713387 11.8656124,11.3526403 L12,11.2676 L12,7.5 C12,6.259091 11.246593,5.19415145 10.1722389,4.73766702 L10,4.67071 L10,7 L6,3.5 L10,0 Z M3,1 C4.10457,1 5,1.89543 5,3 C5,3.68333538 4.65729704,4.28663574 4.13438762,4.6473967 L4,4.73244 L4,11.2676 C4.5978,11.6134 5,12.2597 5,13 C5,14.1046 4.10457,15 3,15 C1.89543,15 1,14.1046 1,13 C1,12.3166462 1.34270296,11.713387 1.86561238,11.3526403 L2,11.2676 L2,4.73244 C1.4022,4.38663 1,3.74028 1,3 C1,1.89543 1.89543,1 3,1 Z"
                                />
                            </svg>
                            <p class="mx-1">Request Merge</p>
                        </Action>
                    </ul>
                </li>
            </ul>
        </div>
    }
}

/// Show Information on a Baseline
#[component]
pub(super) fn BaselineEditor(
    selected: RwSignal<Option<KeyedBaseline>>,
    #[prop(into)] default_language: String,
) -> impl IntoView {
    {
        move || {
            selected.read().map(|sel| {
                let blocks = RwSignal::new(
                    sel.content()
                        .base_corpus()
                        .read()
                        .iter()
                        .enumerate()
                        .map(|(id, b)| EditorBlock {
                            id,
                            inner: b.clone().into(),
                            focus_on_load: false,
                        })
                        .collect::<Vec<_>>(),
                );

                let save_state_action = Action::new(move |blocks: &Vec<EditorBlock>| {
                    let blocks_dehydrated: Vec<Block> =
                        blocks.iter().map(|b| b.inner.clone().into()).collect();
                    async move {
                        sel.content().base_corpus().set(blocks_dehydrated.clone());
                        save_transcription(blocks_dehydrated, sel.id().get()).await?;
                        Ok(())
                    }
                });
                let publish_action = Action::new(move |blocks: &Vec<EditorBlock>| {
                    let blocks_dehydrated: Vec<Block> =
                        blocks.iter().map(|b| b.inner.clone().into()).collect();
                    async move {
                        sel.content().base_corpus().set(blocks_dehydrated.clone());
                        save_transcription(blocks_dehydrated, sel.id().get()).await?;
                        publish_transcription(sel.id().get()).await?;
                        // also close the editor by deselecting this line
                        selected.set(None);
                        Ok(())
                    }
                });

                view! {
                    <EditorWithTabs
                        default_language=default_language.clone()
                        on_save=save_state_action
                        on_publish=publish_action
                        blocks=blocks
                    />
                }
            })
        }
    }
}

/// Save the transcription for an individual line
#[server]
pub async fn save_transcription(blocks: Vec<Block>, line_id: i64) -> Result<(), ServerFnError> {
    use critic_server::auth::AuthSession;
    use leptos_axum::extract;

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
    let config = use_context::<std::sync::Arc<critic_config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;

    // save the fact that this transcription exists to the DB
    Ok(
        match critic_db::save_transcription(&config.db, blocks, line_id, &user.username).await {
            Ok(x) => {
                tracing::debug!("User {} saved a new transcription.", user.username);
                Ok(x)
            }
            Err(e) => {
                tracing::warn!("Failed to save transcription to DB: {e}");
                Err(e)
            }
        }?,
    )
}

/// Publish the transcription for an individual line
///
/// The line may have no ID set yet, in which case the line is created and the id returned
#[server]
pub async fn publish_transcription(line_id: i64) -> Result<(), ServerFnError> {
    use critic_server::auth::AuthSession;
    use leptos_axum::extract;

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
    let config = use_context::<std::sync::Arc<critic_config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;

    // save the fact that this transcription exists to the DB
    Ok(critic_db::publish_transcription(&config.db, line_id, &user.username).await?)
}

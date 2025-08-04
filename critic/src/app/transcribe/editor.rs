//! The components and server functions for the actual transcription view
//!
//! this shows the editor, the publish button, rendering to html and xml and so on

use critic_components::{
    editor::{blocks::EditorBlock, Editor},
    xmleditor::{XmlEditor, XmlState},
};
use critic_format::streamed::Block;
use critic_shared::{
    urls::{IMAGE_BASE_LOCATION, STATIC_BASE_URL},
    ShowHelp,
};
use leptos::{
    either::{Either, EitherOf3},
    prelude::*,
};
use leptos_router::hooks::use_params;

use crate::app::{
    shared::{MsParams, PageParams},
    EmptyError, TopLevelPosition,
};

/// WIP.
/// Get the starting information for this page.
/// If the user has started this transcription already, use that.
/// Otherwise, the initial content is approximated using Basetext-Indexing:
/// - run OCR, find out which text is on this page
/// - find out where the column breaks are, add the relevant basetext column-by-column
/// Result is
///     (Manuscript to initialize the editor with, default-language)
#[server]
async fn get_initial_ms(
    msname: String,
    pagename: String,
) -> Result<(Vec<Block>, String), ServerFnError> {
    use critic_format::streamed::Block;
    use critic_server::{
        auth::AuthSession, db::get_editor_initial_value,
        transcription_store::read_transcription_from_disk,
    };
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
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;
    let initial_seed = get_editor_initial_value(&config.db, &msname, &pagename, &user.username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // TODO: first get default language from the DB
    let default_language = initial_seed.meta.lang;

    if initial_seed.user_has_started {
        Ok((
                read_transcription_from_disk(&config.data_directory, &msname, &pagename, &user.username, &default_language)
                    .map(|(blocks, _pagename)| blocks)
                    .map_err(|e| ServerFnError::new(format!("Transcription /{msname}/{pagename}/{} should exist but is not readable from disk: {e}", user.username)))?,
                default_language))
    } else {
        // TODO - do the whole indexing and find the right place in the base text
        // WIP
        Ok((
            vec![
                Block::Text(critic_format::streamed::Paragraph {
                    lang: "".to_string(),
                    content: "WIP - In the future, the correct part of the basetext will automatically be put here.".to_string()})
            ],
            default_language
        ))
    }
}

#[server]
pub async fn save_transcription(
    blocks: Vec<Block>,
    msname: String,
    pagename: String,
) -> Result<(), ServerFnError> {
    use critic_server::{auth::AuthSession, transcription_store::write_transcription_to_disk};
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
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;

    write_transcription_to_disk(
        blocks,
        &config.data_directory,
        &msname,
        pagename.to_string(),
        &user.username,
    )?;
    // save the fact that this transcription exists to the DB
    critic_server::db::add_transcription(&config.db, &msname, &pagename, &user.username).await?;
    Ok(())
}

#[server]
pub async fn publish_transcription(msname: String, pagename: String) -> Result<(), ServerFnError> {
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
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;

    critic_server::db::publish_transcription(&config.db, &msname, &pagename, &user.username)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

/// The main component for the transcription editor page
#[component]
pub fn TranscribeEditor() -> impl IntoView {
    let set_top_level_pos =
        use_context::<WriteSignal<TopLevelPosition>>().expect("App provides TopLevelPosition");
    *set_top_level_pos.write() = TopLevelPosition::Transcribe;

    let ms_param = use_params::<MsParams>();
    let page_param = use_params::<PageParams>();

    // get msname from url
    let both_names = move || {
        (
            ms_param
                .read_untracked()
                .as_ref()
                .ok()
                .and_then(|x| x.msname.clone()),
            page_param
                .read_untracked()
                .as_ref()
                .ok()
                .and_then(|x| x.pagename.clone()),
        )
    };
    // get initial state from the server
    let blocks_res = Resource::new(both_names, async |(ms_name_opt, page_name_opt)| {
        if let (Some(x), Some(y)) = (ms_name_opt, page_name_opt) {
            get_initial_ms(x, y).await
        } else {
            Err(ServerFnError::new(
                "Did not get both Manuscript and Page name to fetch initial data.",
            ))
        }
    });

    view! {
        <div class="flex h-full flex-col">
            <h1 class="p-10 text-center text-6xl font-semibold">
                "Transcribing "{move || both_names().1}
            </h1>
            // show links to the image
            <ErrorBoundary fallback=|_errors| {
                view! { "Failed to get manuscriptname and page name from the url." }
            }>
                {move || {
                    if let (Some(msname), Some(pagename)) = (
                        ms_param.get().map(|p| p.msname).unwrap_or(None),
                        page_param.get().map(|p| p.pagename).unwrap_or(None),
                    ) {
                        let image_link = format!(
                            "{STATIC_BASE_URL}{IMAGE_BASE_LOCATION}/{msname}/{pagename}/original.webp",
                        );
                        Ok(
                            view! {
                                <div class="flex justify-center">
                                    <a
                                        class="text-md m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50 hover:bg-slate-500"
                                        href=image_link.clone()
                                    >
                                        View the image
                                    </a>
                                    <a
                                        class="text-md m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50 hover:bg-slate-500"
                                        href=image_link
                                        download="pagename"
                                    >
                                        Download the image
                                    </a>
                                </div>
                            },
                        )
                    } else {
                        Err(EmptyError {})
                    }
                }}
            </ErrorBoundary>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div>
                        "Error: failed to get initial data for the Transcription editor"
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
                    view! { <p>"Loading manuscripts..."</p> }
                }>
                    {move || {
                        blocks_res
                            .get()
                            .map(|blocks_or_err| {
                                blocks_or_err
                                    .map(|(blocks, default_lang)| {
                                        let blocks = RwSignal::new(
                                            blocks
                                                .into_iter()
                                                .enumerate()
                                                .map(|(id, b)| EditorBlock {
                                                    id,
                                                    inner: b.into(),
                                                    focus_on_load: false,
                                                })
                                                .collect::<Vec<_>>(),
                                        );
                                        let save_state_action = Action::new(move |
                                            blocks: &Vec<EditorBlock>|
                                        {
                                            let blocks_dehydrated = blocks
                                                .iter()
                                                .map(|b| b.inner.clone().into())
                                                .collect();
                                            async move {
                                                if let (Some(msname), Some(pagename)) = both_names() {
                                                    save_transcription(blocks_dehydrated, msname, pagename)
                                                        .await
                                                } else {
                                                    Ok(())
                                                }
                                            }
                                        });
                                        let publish_action = Action::new(move |
                                            blocks: &Vec<EditorBlock>|
                                        {
                                            let blocks_dehydrated = blocks
                                                .iter()
                                                .map(|b| b.inner.clone().into())
                                                .collect();
                                            async move {
                                                if let (Some(msname), Some(pagename)) = both_names() {
                                                    save_transcription(
                                                            blocks_dehydrated,
                                                            msname.clone(),
                                                            pagename.clone(),
                                                        )
                                                        .await?;
                                                    publish_transcription(msname, pagename).await
                                                } else {
                                                    Ok(())
                                                }
                                            }
                                        });
                                        both_names()
                                            .1
                                            .map(|pagename| {
                                                view! {
                                                    <EditorWithTabs
                                                        blocks=blocks
                                                        default_language=default_lang
                                                        on_save=save_state_action
                                                        on_publish=publish_action
                                                        pagename=pagename
                                                    />
                                                }
                                            })
                                    })
                            })
                    }}
                </Transition>
            </ErrorBoundary>
        </div>
    }
}

const SHORTCUT_DESCRIPTIONS: &[(&str, &str, &str)] = &[
    (
        "s",
        "Save",
        "Save the current state of the editor to the server",
    ),
    ("z", "Undo", "Undo your last action"),
    ("r", "Redo", "Redo the action you just undid"),
    ("t", "Text", "Add a new block of text without markup"),
    (
        "a",
        "Abbreviation",
        "Turn the selection into an abbreviation",
    ),
    ("u", "Uncertain", "Mark the selection as uncertain"),
    ("l", "Lacuna", "Mark the selection as lacunous"),
    ("c", "Correction", "Mark the selection as corrected"),
    (
        "v",
        "Verse",
        "Delete the selection, putting a verse boundary in its place",
    ),
    (
        "<space>",
        "Space",
        "Delete the selection, marking intended whitespace",
    ),
    (
        "<enter>",
        "Enter",
        "Delete the selection, marking the end of a line or column",
    ),
    ("c", "Check", "XML only: check that XML is valid."),
];

#[component]
fn HelpOverlay(active: RwSignal<ShowHelp>) -> impl IntoView {
    view! {
        <div
            on:click=move |_| { active.update(|a| a.set_off()) }
            // my tailwind is not compiling backdrop-blur-xs and I don't know why..
            class="absolute inset-0 w-full bg-slate-900/90 backdrop-blur-[8px] overflow-y-auto"
            class=("block", move || active.read().get())
            class=("hidden", move || !active.read().get())
        >
            <div class="absolute left-20 w-4/5 text-lg text-white">
                <p>
                    "This is the transcription editor. Copy a base text from another edition, then edit it here, marking up differences you find in the manuscript image."
                </p>
                <p>
                    "You can use the normal Editor, view an approximated render of what you have entered so far, or edit the XML directly. Remember that when you edit XML, you need to convert it to the normal editor before saving or publishing to make sure the data is correct."
                </p>
                <p>
                    "You can use these keyboard shortcuts: "
                    <span class="text-2xl">ctrl + alt +</span>"..."
                </p>
                <table class="table-fixed flex justify-around">
                    <tbody>
                        {SHORTCUT_DESCRIPTIONS
                            .iter()
                            .map(|(key, name, descr)| {
                                view! {
                                    <tr>
                                        <td class="text-2xl w-28">{*key}</td>
                                        <td class="text-xl w-36">{*name}</td>
                                        <td>{*descr}</td>
                                    </tr>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorTabs {
    Block,
    Render,
    Xml,
}

/// Switches between the different tabs in the editor
#[component]
fn EditorWithTabs(
    blocks: RwSignal<Vec<EditorBlock>>,
    default_language: String,
    on_save: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
    on_publish: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
    pagename: String,
) -> impl IntoView {
    let help_active: RwSignal<ShowHelp> = use_context().expect("Root mounts ShowHelp context");
    let tab_active = RwSignal::new(EditorTabs::Block);

    let xml_state = RwSignal::new(XmlState::Checked);

    view! {
        <div class="mx-16 my-4 flex flex-col h-full bg-slate-800 relative">
            <HelpOverlay active=help_active />
            <div class="text-red">
                {move || match xml_state.get() {
                    XmlState::Checked | XmlState::Unchecked => Either::Left(()),
                    XmlState::Err(e) => Either::Right(view! { <p>{e}</p> }),
                }}
            </div>
            <div id="editor-tab-header" class="mb-4 p-2 pb-0 border-b border-slate-600">
                <button
                    on:click=move |_| {
                        match xml_state.get() {
                            XmlState::Checked => {
                                tab_active.set(EditorTabs::Block);
                            }
                            XmlState::Err(_) => {}
                            XmlState::Unchecked => {
                                xml_state
                                    .set(
                                        XmlState::Err(
                                            "You need to check the XML first.".to_string(),
                                        ),
                                    );
                            }
                        }
                    }
                    class="mx-2 mb-0 p-2 hover:bg-slate-500 rounded-t-lg"
                    class=("bg-sky-600/30", move || tab_active.get() == EditorTabs::Block)
                >
                    Editor
                </button>
                <button
                    on:click=move |_| {
                        match xml_state.get() {
                            XmlState::Checked => {
                                tab_active.set(EditorTabs::Render);
                            }
                            XmlState::Err(_) => {}
                            XmlState::Unchecked => {
                                xml_state
                                    .set(
                                        XmlState::Err(
                                            "You need to check the XML first.".to_string(),
                                        ),
                                    );
                            }
                        }
                    }
                    class="mx-2 mb-0 p-2 hover:bg-slate-500 rounded-t-lg"
                    class=("bg-sky-600/30", move || tab_active.get() == EditorTabs::Render)
                >
                    Render
                </button>
                <button
                    on:click=move |_| {
                        tab_active.set(EditorTabs::Xml);
                    }
                    class="mx-2 mb-0 p-2 hover:bg-slate-500 rounded-t-lg"
                    class=("bg-sky-600/30", move || tab_active.get() == EditorTabs::Xml)
                >
                    XML
                </button>
            </div>
            {move || {
                tab_active
                    .with(|tab| match tab {
                        EditorTabs::Block => {
                            let lang_cloned = default_language.clone();
                            EitherOf3::A(
                                view! {
                                    <Editor
                                        blocks=blocks
                                        default_language=lang_cloned
                                        on_save=on_save
                                    />
                                },
                            )
                        }
                        EditorTabs::Render => {
                            EitherOf3::B(
                                view! {
                                    "TODO. In the future, you will see an approximate render of your entered text as it would look like on the page."
                                },
                            )
                        }
                        EditorTabs::Xml => {
                            EitherOf3::C(
                                view! {
                                    <XmlEditor
                                        blocks=blocks
                                        on_save=on_save
                                        xml_state=xml_state
                                        pagename=pagename.clone()
                                        default_language=default_language.clone()
                                    />
                                },
                            )
                        }
                    })
            }}
        </div>
        // note: deliberately not in the div, but after it
        <div class="flex justify-center w-full">
            {move || {
                xml_state
                    .with(|state| match state {
                        XmlState::Checked => {
                            Either::Left(
                                view! {
                                    <button
                                        class="w-96 text-2xl m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50 shadow-sm shadow-sky-600 hover:bg-slate-500"
                                        on:click=move |_| {
                                            on_publish.dispatch(blocks.get());
                                        }
                                    >
                                        "Publish this transcription"
                                    </button>
                                },
                            )
                        }
                        XmlState::Unchecked => {
                            Either::Right(
                                view! {
                                    <span class="w-96 text-2xl m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50">
                                        "Check your XML before publishing!"
                                    </span>
                                },
                            )
                        }
                        XmlState::Err(_) => {
                            Either::Right(
                                view! {
                                    <span class="w-96 text-2xl m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50">
                                        "Fix errors before publishing!"
                                    </span>
                                },
                            )
                        }
                    })
            }}
        </div>
    }
}

//! The components and server functions for the actual transcription view
//!
//! this shows the editor, the publish button, rendering to html and xml and so on

use critic_components::editor::{blocks::EditorBlock, Editor};
use critic_format::streamed::{Block, Manuscript};
use leptos::prelude::*;
use leptos_router::hooks::use_params;

use crate::app::{
    shared::{MsParams, PageParams},
    TopLevelPosition,
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
) -> Result<(Manuscript, String), ServerFnError> {
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

    if initial_seed.user_has_started {
        Ok((
                read_transcription_from_disk(&config.data_directory, &msname, &pagename, &user.username)
                    .map_err(|e| ServerFnError::new(format!("Transcription /{msname}/{pagename}/{} should exist but is not readable from disk: {e}", user.username)))?,
                initial_seed.meta.lang))
    } else {
        // TODO - do the whole indexing and find the right place in the base text
        // WIP
        Ok((
                Manuscript {
                    meta: critic_format::normalized::Meta {
                        name: format!("{msname} page {pagename}"),
                        page_nr: pagename,
                        title: msname,
                        institution: initial_seed.meta.institution,
                        collection: initial_seed.meta.collection,
                        hand_desc: initial_seed.meta.hand_desc,
                        script_desc: initial_seed.meta.script_desc},
                        content: vec![Block::Text(critic_format::streamed::Paragraph {
                            lang: initial_seed.meta.lang.clone(),
                            content: "WIP - In the future, the correct part of the basetext will automatically be put here.".to_string()
                        })]
                },
                initial_seed.meta.lang))
    }
}

#[server]
pub async fn save_transcription(
    blocks: Vec<Block>,
    meta: critic_format::streamed::Meta,
) -> Result<(), ServerFnError> {
    use critic_format::streamed::Manuscript;
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

    // save the data to disk
    let ms = Manuscript {
        meta,
        content: blocks,
    };
    // TODO this is really ugly. It would be nice if writing to XML could take the MS by ref (since
    // we have to seralize it anyways, this does not need to own any of the actual data)
    //
    // However, that would require making the type have a lifetime into the data, and I do not have
    // enough time to set this up right now.
    let msname = ms.meta.title.clone();
    let pagename = ms.meta.page_nr.clone();
    write_transcription_to_disk(ms, &config.data_directory, &user.username)?;
    // save the fact that this transcription exists to the DB
    critic_server::db::add_transcription(&config.db, &msname, &pagename, &user.username).await?;
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
            ms_param.read().as_ref().ok().and_then(|x| x.msname.clone()),
            page_param
                .read()
                .as_ref()
                .ok()
                .and_then(|x| x.pagename.clone()),
        )
    };
    // get initial state from the server
    let ms_res = Resource::new(both_names, async |(ms_name_opt, page_name_opt)| {
        if let (Some(x), Some(y)) = (ms_name_opt, page_name_opt) {
            get_initial_ms(x, y).await
        } else {
            Err(ServerFnError::new(
                "Did not get both Manuscript and Page name to fetch initial data.",
            ))
        }
    });

    view! {
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
                    ms_res
                        .get()
                        .map(|ms_or_err| {
                            ms_or_err
                                .map(|(manuscript, default_lang)| {
                                    let blocks = RwSignal::new(
                                        manuscript
                                            .content
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
                                        let cloned_meta = manuscript.meta.clone();
                                        async move {
                                            save_transcription(blocks_dehydrated, cloned_meta).await
                                        }
                                    });
                                    view! {
                                        <Editor
                                            blocks=blocks
                                            default_language=default_lang
                                            on_save=save_state_action
                                        />
                                    }
                                })
                        })
                }}
            </Transition>
        </ErrorBoundary>
    }
}

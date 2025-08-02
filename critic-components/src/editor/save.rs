//! Functions for saving (and loading) the state from the server.

use critic_format::streamed::Block;
use leptos::prelude::*;

// #[server]
// pub(super) async fn load_editor_state() -> Result<critic_format::streamed::Manuscript, ServerFnError>
// {
//     use std::path::Path;
//
//     let path = Path::new("tmp/data");
//     let file = std::fs::File::open(path).map_err(|e| ServerFnError::new(e.to_string()))?;
//     let buf_reader = std::io::BufReader::new(file);
//     let ds: critic_format::schema::Tei =
//         quick_xml::de::from_reader(buf_reader).map_err(|e| ServerFnError::new(e.to_string()))?;
//     let normalized: critic_format::normalized::Manuscript =
//         ds.try_into()
//             .map_err(|e: critic_format::denorm::NormalizationError| {
//                 ServerFnError::new(e.to_string())
//             })?;
//     let streamed: critic_format::streamed::Manuscript = normalized
//         .try_into()
//         .map_err(|e: critic_format::destream::StreamError| ServerFnError::new(e.to_string()))?;
//
//     Ok(streamed)
// }
//
// /// We take streamed blocks because they have no Signals and so can properly (de-)serialize
// #[server]
// pub(super) async fn save_editor_state(
//     blocks: Vec<critic_format::streamed::Block>,
// ) -> Result<(), ServerFnError> {
//     use std::io::Write;
//     use std::path::Path;
//
//     let path = Path::new("tmp/data");
//     let mut file = std::fs::File::create(path).map_err(|e| ServerFnError::new(e.to_string()))?;
//
//     // todo fill actual data
//     let ms = critic_format::streamed::Manuscript {
//         meta: critic_format::normalized::Meta {
//             name: "name".to_string(),
//             page_nr: "pgnr".to_string(),
//             title: "title".to_string(),
//             institution: None,
//             collection: None,
//             hand_desc: None,
//             script_desc: None,
//         },
//         content: blocks,
//     };
//
//     let destreamed: critic_format::normalized::Manuscript = ms
//         .try_into()
//         .map_err(|e: critic_format::destream::StreamError| ServerFnError::new(e.to_string()))?;
//     let denormed: critic_format::schema::Tei =
//         destreamed
//             .try_into()
//             .map_err(|e: critic_format::denorm::NormalizationError| {
//                 ServerFnError::new(e.to_string())
//             })?;
//     let sr = quick_xml::se::to_string_with_root("TEI", &denormed)
//         .map_err(|e| ServerFnError::new(e.to_string()))?;
//     file.write(sr.as_bytes())
//         .map_err(|e| ServerFnError::new(e.to_string()))?;
//     Ok(())
// }

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

//! Functions for saving (and loading) the state from the server.

use leptos::prelude::ServerFnError;
use leptos::prelude::*;

/// TODO do these properly with writing and getting functions in critic-format
#[server]
pub(super) async fn load_editor_state() -> Result<critic_format::streamed::Manuscript, ServerFnError>
{
    use std::path::Path;

    let path = Path::new("tmp/data");
    let file = std::fs::File::open(path).map_err(|e| ServerFnError::new(e.to_string()))?;
    let buf_reader = std::io::BufReader::new(file);
    let ds: critic_format::schema::Tei =
        quick_xml::de::from_reader(buf_reader).map_err(|e| ServerFnError::new(e.to_string()))?;
    let normalized: critic_format::normalized::Manuscript =
        ds.try_into()
            .map_err(|e: critic_format::denorm::NormalizationError| {
                ServerFnError::new(e.to_string())
            })?;
    let streamed: critic_format::streamed::Manuscript = normalized
        .try_into()
        .map_err(|e: critic_format::destream::StreamError| ServerFnError::new(e.to_string()))?;

    Ok(streamed)
}

/// We take streamed blocks because they have no Signals and so can properly (de-)serialize
#[server]
pub(super) async fn save_editor_state(
    blocks: Vec<critic_format::streamed::Block>,
) -> Result<(), ServerFnError> {
    use std::io::Write;
    use std::path::Path;

    let path = Path::new("tmp/data");
    let mut file = std::fs::File::create(path).map_err(|e| ServerFnError::new(e.to_string()))?;

    // todo fill actual data
    let ms = critic_format::streamed::Manuscript {
        meta: critic_format::normalized::Meta {
            name: "name".to_string(),
            page_nr: "pgnr".to_string(),
            title: "title".to_string(),
            institution: None,
            collection: None,
            hand_desc: None,
            script_desc: None,
        },
        content: blocks,
    };

    let destreamed: critic_format::normalized::Manuscript = ms
        .try_into()
        .map_err(|e: critic_format::destream::StreamError| ServerFnError::new(e.to_string()))?;
    let denormed: critic_format::schema::Tei =
        destreamed
            .try_into()
            .map_err(|e: critic_format::denorm::NormalizationError| {
                ServerFnError::new(e.to_string())
            })?;
    let sr = quick_xml::se::to_string_with_root("TEI", &denormed)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    file.write(sr.as_bytes())
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

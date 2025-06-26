//! Functions for saving (and loading) the state from the server.

use std::fs::read_to_string;

use leptos::prelude::ServerFnError;
use leptos::prelude::*;

use super::EditorBlock;

/// TODO do these properly with writing and getting functions in critic-format
//#[server]
pub(super) async fn load_editor_state() -> Result<Vec<EditorBlock>, ServerFnError> {
    use std::path::Path;

    let path = Path::new("tmp/data");
    let file = std::fs::File::open(path).map_err(|e| ServerFnError::new(e.to_string()))?;
    let buf_reader = std::io::BufReader::new(file);
    let ds: critic_format::schema::Tei = quick_xml::de::from_reader(buf_reader).map_err(|e| ServerFnError::new(e.to_string()))?;
    let normalized: critic_format::normalized::Manuscript = ds.try_into().map_err(|e: critic_format::denorm::NormalizationError| ServerFnError::new(e.to_string()))?;
    let streamed: critic_format::streamed::Manuscript = normalized.try_into().map_err(|e: critic_format::destream::StreamError| ServerFnError::new(e.to_string()))?;
    let blocks = streamed.content.into_iter().enumerate().map(|(idx, x)| EditorBlock { focus_on_load: false, inner: x.into(), id: idx as i32, }).collect();

    Ok(blocks)
}

#[server]
pub(super) async fn save_editor_state(
    blocks: Vec<super::EditorBlock>,
) -> Result<(), ServerFnError> {
    use std::io::Write;
    use std::path::Path;
    use tracing::info;

    let path = Path::new("tmp/data");
    let mut file = std::fs::File::create(path).map_err(|e| ServerFnError::new(e.to_string()))?;

    let streamed: Vec<critic_format::streamed::Block> =
        blocks.into_iter().map(|x| x.inner.into()).collect();
    let ms = critic_format::streamed::Manuscript {
        meta: critic_format::normalized::Meta {
            name: "name".to_string(),
            page_nr: "pgnr".to_string(),
            title: "title".to_string(),
            institution: None,
            collection: None,
            hand_desc: "handDesc".to_string(),
            script_desc: "scriptDesc".to_string(),
        },
        content: streamed,
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

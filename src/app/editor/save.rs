//! Functions for saving (and loading) the state from the server.

use std::io::Read;

use leptos::prelude::ServerFnError;
use leptos::prelude::*;

use super::EditorBlockDry;

#[server]
pub(super) async fn load_editor_state() -> Result<Vec<EditorBlockDry>, ServerFnError> {
    use std::path::Path;

    let path = Path::new("tmp/data");
    let file = std::fs::File::open(path).map_err(|e| ServerFnError::new(e.to_string()))?;
    let blocks: Vec<EditorBlockDry> =
        serde_json::from_reader(file).map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(blocks)
}

#[server]
pub(super) async fn save_editor_state(blocks: Vec<EditorBlockDry>) -> Result<(), ServerFnError> {
    use std::io::Write;
    use std::path::Path;

    let path = Path::new("tmp/data");
    let mut file = std::fs::File::create(path).map_err(|e| ServerFnError::new(e.to_string()))?;
    file.write_all(&serde_json::to_vec(&blocks).map_err(|e| ServerFnError::new(e.to_string()))?)
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

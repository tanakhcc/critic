//! The components and server functions for the actual transcription view
//!
//! this shows the editor, the publish button, rendering to html and xml and so on

use leptos::prelude::*;

use crate::app::TopLevelPosition;

/// The main component for the transcription editor page
#[component]
pub fn TranscribeEditor() -> impl IntoView {
    let set_top_level_pos = use_context::<WriteSignal<TopLevelPosition>>().expect("App provides TopLevelPosition");
    *set_top_level_pos.write() = TopLevelPosition::Transcribe;
}

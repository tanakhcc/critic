//! The XML editor (Basically a glorified textarea.

use critic_format::{
    from_xml,
    streamed::{Manuscript, Meta},
    to_xml,
};
use leptos::{either::Either, ev::keydown, prelude::*};
use leptos_use::use_event_listener;

use crate::{editor::blocks::EditorBlock, DEFAULT_BUTTON_CLASSES};

#[derive(Debug, Clone)]
pub enum XmlState {
    /// We know that the XML state is currently OK
    Checked,
    /// We know that the XML state is currently BAD
    Err(String),
    /// We have to check first
    Unchecked,
}

/// The XML Editor.
///
/// Can:
/// - edit the raw xml
/// - check correctness
/// - save the xml by first checking/converting and then calling on_save
///     - this checks, sets xml_error to None if that worked, then
#[component]
pub fn XmlEditor(
    blocks: RwSignal<Vec<EditorBlock>>,
    meta: Meta,
    on_save: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
    /// Communicates the XML state to the parent (we disallow leaving the XmlEditor if the XMl is
    /// invalid
    xml_state: RwSignal<XmlState>,
) -> impl IntoView {
    let ms = Manuscript {
        content: blocks
            .get_untracked()
            .into_iter()
            .map(|b| b.inner.into())
            .collect(),
        meta,
    };
    let starting_xml = match to_xml(ms) {
        Ok(x) => x,
        Err(e) => {
            return Either::Left(view! {
                <p>
                    "Could not convert from blocks to XML: "{move || e.to_string()}
                    ". Please go back to block editor and fix that problem."
                </p>
            });
        }
    };

    let textarea_content = RwSignal::new(starting_xml);

    let check = move || {
        match from_xml(textarea_content.read().as_bytes()).map_err(|e| e.to_string()) {
            Ok(ms) => {
                // check was ok
                *xml_state.write() = XmlState::Checked;
                // set blocks accordingly
                *blocks.write() = ms
                    .content
                    .into_iter()
                    .enumerate()
                    .map(|(id, b)| EditorBlock {
                        inner: b.into(),
                        id,
                        focus_on_load: false,
                    })
                    .collect();
                true
            }
            Err(e) => {
                // check was bad
                *xml_state.write() = XmlState::Err(e);
                false
            }
        }
    };
    let save = move || {
        if check() {
            // and now save
            on_save.dispatch(blocks.get());
        };
    };

    let textarea_ref = NodeRef::new();
    let _cleanup = use_event_listener(textarea_ref, keydown, move |evt| {
        // <ctrl>-<alt>-S - Save
        if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 83 {
            save();
        // <ctrl>-<alt>-C - Check
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 67 {
            check();
        }
    });

    Either::Right(view! {
        <div id="xml-editor">
            <textarea
                node_ref=textarea_ref
                id="xml-edit-content"
                class="m-3 p-1 bg-slate-700"
                rows=13
                cols=90
                prop:value=move || textarea_content.get()
                autocomplete="false"
                spellcheck="false"
                on:input:target=move |ev| {
                    *textarea_content.write() = ev.target().value();
                    xml_state.set(XmlState::Unchecked);
                }
            />
            <div>
                <button
                    on:click=move |_| {
                        check();
                    }
                    class=DEFAULT_BUTTON_CLASSES
                >
                    Check
                </button>
                <button
                    on:click=move |_| {
                        save();
                    }
                    class=DEFAULT_BUTTON_CLASSES
                >
                    Save
                </button>
            </div>
        </div>
    })
}

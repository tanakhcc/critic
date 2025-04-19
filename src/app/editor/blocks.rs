//! The Types and associated functions for Blocks
//!
//! This module defines, what blocks are available, do and look like. Interaction with other
//! elements is handled in [`editor`](crate::app::editor) itself.

use leptos::prelude::*;
use web_sys::HtmlInputElement;


#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) struct EditorBlock {
    id: i32,
    inner: InnerBlock,
    focus_on_load: bool,
}
impl EditorBlock {
    pub fn new(id: i32, block_type: InnerBlockType, content: String, focus_on_load: bool) -> Self {
        Self {
            id,
            inner: InnerBlock::new_from_type_and_content(block_type, content),
            focus_on_load,
        }
    }

    pub fn id(&self) -> i32 {
        self.id
    }

    pub(super) fn view(self) -> impl IntoView {
        view!{
            <span>{self.id}":"{move || self.inner.clone().view(self.id, self.focus_on_load)}</span>
        }
    }

    /// Split this block, returning new blocks and the index of the block which defaults as the
    /// newly inserted one.
    pub(super) fn split_at_selection(
        &self,
        start: usize,
        end: usize,
        new_block_type: InnerBlockType,
        new_index: &mut i32,
    ) -> Vec<EditorBlock> {
        // add the ids to the inner blocks created from splitting this inner block
        self.inner
            .split_at_selection(start, end, new_block_type)
            .into_iter()
            .map(|iblck| {
                let block = EditorBlock {
                    id: *new_index,
                    inner: iblck.0,
                    focus_on_load: iblck.1,
                };
                *new_index += 1;
                return block;
            })
            .collect()
    }
}
/// Dataless types for Blocks
pub(super) enum InnerBlockType {
    /// Raw text without special markup
    Text,
    /// A part of Text with uncertainty
    ///
    /// These are (sequences of) glyphs where the intention is not clear
    Uncertain,
    /// A part of Text that is absent or entirely unreadable
    ///
    /// These are places that can only be supplied, no actual reading of the remains is possible
    Lacuna,
    /// A break (Line, Column, Page, ...)
    Break,
}
/// Block type with data
///
/// TODO: make this a struct, holding type, content and metadata instead
/// then revisit the diverse methods here and every other match statement over type
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub(super) enum InnerBlock {
    /// Raw text without special markup
    /// text
    Text(RwSignal<String>),
    /// A part of Text with uncertainty
    /// (proposed-text, reason)
    Uncertain(RwSignal<String>, RwSignal<String>),
    /// A part of Text that is absent or entirely unreadable
    /// (proposed-text, reason)
    Lacuna(RwSignal<String>, RwSignal<String>),
    /// A break (Line, Column, Page, ...)
    /// TODO: we want this to be an enum over type instead; with selection menu in GUI
    /// (type of break)
    Break(RwSignal<String>),
}
impl InnerBlock {
    /// Create a new Block with content
    pub fn new_from_type_and_content(block_type: InnerBlockType, content: String) -> Self {
        match block_type {
            InnerBlockType::Text => InnerBlock::Text(RwSignal::new(content)),
            InnerBlockType::Uncertain => {
                InnerBlock::Uncertain(RwSignal::new(content), RwSignal::<String>::default())
            }
            InnerBlockType::Lacuna => {
                InnerBlock::Lacuna(RwSignal::new(content), RwSignal::<String>::default())
            }
            InnerBlockType::Break=> {
                // Breaks do not have content; ignore it
                InnerBlock::Break(RwSignal::<String>::default())
            }
        }
    }

    /// Create a new Block without content
    pub(super) fn new_from_type(block_type: InnerBlockType) -> Self {
        Self::new_from_type_and_content(block_type, "".to_owned())
    }

    pub(super) fn view(self, id: i32, do_focus: bool) -> impl IntoView {
        let focus_element = NodeRef::new();
        // if do_focus is true, focus this input when it is created
        if do_focus {
            Effect::new(move |_| {
                focus_element.on_load(|input: HtmlInputElement| {
                    let _ = input.focus();
                });
            });
        }
        match self {
            InnerBlock::Text(content) => {
                view! {
                    <div>
                        <p>"Raw Text: "</p>
                        <input node_ref=focus_element id={format!("block-input-{id}")} value=content.get() on:input:target=move |ev| {
                            content.set(ev.target().value());
                        }/>
                    </div>
                }.into_any()
            }
            InnerBlock::Uncertain(content, reason) => {
                view! {
                    <div>
                        <p>"Uncertain: "</p>
                        <input id={format!("block-input-{id}")} value=content.get() on:input:target=move |ev| {
                            content.set(ev.target().value());
                        }/>
                        // we want to focus on the uncertainty for a new uncertain passage
                        // it is most likely that someone took a part of Text and marked a part as
                        // uncertain. In this case, the main content is already correct but the reasons
                        // needs to be supplied next
                        <input node_ref=focus_element value=reason.get() on:input:target=move |ev| {
                            reason.set(ev.target().value());
                        }/>
                    </div>
                }.into_any()
            }
            InnerBlock::Lacuna(content, reason) => {
                view! {
                    <div>
                        <p>"Lacuna: "</p>
                        <input id={format!("block-input-{id}")} value=content.get() on:input:target=move |ev| {
                            content.set(ev.target().value());
                        }/>
                        // we want to focus on the reason for a new lacunous passage
                        // it is most likely that someone took a part of Text and marked a part as
                        // lacuna. In this case, the main content is already correct but the reasons
                        // needs to be supplied next
                        <input node_ref=focus_element value=reason.get() on:input:target=move |ev| {
                            reason.set(ev.target().value());
                        }/>
                    </div>
                }.into_any()
            }
            InnerBlock::Break(break_type) => {
                view! {
                    <div>
                        <p>"Break: "</p>
                        // TODO make this a drop down instead
                        <input node_ref=focus_element value=break_type.get() on:input:target=move |ev| {
                            break_type.set(ev.target().value());
                        }/>
                    </div>
                }.into_any()
            }
        }
    }

    /// Split this block, returning new blocks and the index of the block which defaults as the
    /// newly inserted one.
    ///
    /// Returns a vec of InnerBlock, focus_on_load
    fn split_at_selection(
        &self,
        start: usize,
        end: usize,
        new_block_type: InnerBlockType,
    ) -> Vec<(InnerBlock, bool)> {
        let complete_value = match self {
            Self::Text(el) => el.get(),
            Self::Uncertain(el, _) => el.get(),
            Self::Lacuna(el, _) => el.get(),
            Self::Break(_) => "".to_owned(),
        };
        let (before_part, new_part, after_part) = if start == 0 {
            if end == complete_value.len() {
                // everything selected - do nothing
                (None, complete_value.as_ref(), None)
            } else {
                // create a new node before
                (None, &complete_value[0..end], Some(&complete_value[end..]))
            }
        } else {
            if end == complete_value.len() {
                // create a new node after
                (
                    Some(&complete_value[0..start]),
                    &complete_value[start..],
                    None,
                )
            } else {
                // split in three
                (
                    Some(&complete_value[..start]),
                    &complete_value[start..end],
                    Some(&complete_value[end..]),
                )
            }
        };
        let mut res = vec![];
        // first and last block (if any) keeps the same type as this one
        if let Some(content) = before_part {
            res.push((
                match self {
                    Self::Text(_) => InnerBlock::Text(RwSignal::new(content.to_owned())),
                    Self::Uncertain(_, y) => {
                        InnerBlock::Uncertain(RwSignal::new(content.to_owned()), *y)
                    }
                    Self::Lacuna(_, y) => InnerBlock::Lacuna(RwSignal::new(content.to_owned()), *y),
                    Self::Break(y) => InnerBlock::Break(*y),
                },
                false,
            ));
        };
        res.push((
            InnerBlock::new_from_type_and_content(new_block_type, new_part.to_owned()),
            // we do want to autofocus on the middle block
            true,
        ));
        if let Some(content) = after_part {
            res.push((
                match self {
                    Self::Text(_) => InnerBlock::Text(RwSignal::new(content.to_owned())),
                    Self::Uncertain(_, y) => {
                        InnerBlock::Uncertain(RwSignal::new(content.to_owned()), *y)
                    }
                    Self::Lacuna(_, y) => InnerBlock::Lacuna(RwSignal::new(content.to_owned()), *y),
                    Self::Break(y) => InnerBlock::Break(*y),
                },
                false,
            ));
        };
        return res;
    }
}


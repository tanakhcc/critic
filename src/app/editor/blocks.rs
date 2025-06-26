//! The Types and associated functions for Blocks
//!
//! This module defines, what blocks are available, do and look like. Interaction with other
//! elements is handled in [`editor`](crate::app::editor) itself.

use critic_format::streamed::{
    Block, BlockType, BreakType, FromTypeLangAndContent, Lacuna, Paragraph, Uncertain,
};
use leptos::{html::Textarea, prelude::*};
use serde::{Deserialize, Serialize};

use super::{UnReStack, UnReStep};

const TEXTAREA_DEFAULT_ROWS: i32 = 2;
const TEXTAREA_DEFAULT_COLS: i32 = 30;

/// A single block that we change in the editor
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(super) struct EditorBlock {
    /// ID of the block (i.e. creation-order, NOT position)
    pub id: i32,
    /// The actual content
    pub inner: InnerBlock,
    /// Should this block focus when loaded?
    pub focus_on_load: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Text {
    content: RwSignal<String>,
    lang: RwSignal<String>,
}

fn inner_text_view(undo_stack: RwSignal<UnReStack>, text: Text, focus_element: leptos::prelude::NodeRef<Textarea>, id: i32) -> impl IntoView {
            // initialize the old content with the current one
            let (old_content, set_old_content) = signal(text.content.get_untracked());
            let (old_lang, set_old_lang) = signal(text.lang.get_untracked());
            view! {
                <div>
                    <p
                        class="font-light text-xs">
                        "Raw Text: "
                    </p>
                    <input
                    prop:value=text.lang
                    class="text-sm"
                    placeholder="reason"
                    autocomplete="false"
                    spellcheck="false"
                    on:input:target=move |ev| {
                        text.lang.set(ev.target().value());
                    }
                    on:change:target=move |ev| {
                        let current_old_lang = old_lang.get();
                        let new_lang = ev.target().value();
                        set_old_lang.set(new_lang.clone());
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Text(Paragraph { lang: current_old_lang, content: text.content.get_untracked() }),
                                Block::Text(Paragraph { lang: new_lang, content: text.content.get_untracked() }),
                                ));
                    }/>
                    <textarea
                    class="bg-yellow-100 text-black font-mono"
                    id={format!("block-input-{id}")}
                    node_ref=focus_element
                    autocomplete="false"
                    spellcheck="false"
                    rows=TEXTAREA_DEFAULT_ROWS
                    cols=TEXTAREA_DEFAULT_COLS
                    prop:value=text.content
                    on:input:target=move |ev| {
                        //change the current content when updated
                        text.content.set(ev.target().value());
                    }
                    on:change:target=move |ev| {
                        // the input is unfocused - we now want to add something to the undo
                        // machine
                        // the content that was last saved (on last unfocus of this element)
                        let current_old_content = old_content.get();
                        // current real value
                        let new_content = ev.target().value();
                        // save the new content on this unfocus (for the next run of this
                        // closure)
                        set_old_content.set(new_content.clone());
                        // add the diff between the last unfocus and this unfocus to the stack
                        undo_stack.write().push_undo(UnReStep::new_data_change(
                                id,
                                Block::Text(Paragraph {
                                    lang: text.lang.get_untracked(),
                                    content: current_old_content }),
                                Block::Text(Paragraph {
                                    lang: text.lang.get_untracked(),
                                    content: new_content }
                            )));
                    }
                />
                </div>
            }
}

#[component]
fn InnerView(inner: InnerBlock, id: i32, focus_on_load: bool) -> impl IntoView {
    let focus_element = NodeRef::<Textarea>::new();
    // if do_focus is true, focus this input when it is created
    if focus_on_load {
        Effect::new(move |_| {
            focus_element.on_load(|input: web_sys::HtmlTextAreaElement| {
                let _ = input.focus();
            });
        });
    }

    let undo_stack = use_context::<RwSignal<UnReStack>>()
        .expect("Blocks need to be nested in an editor providing an undo stack");

    match inner {
        InnerBlock::Text(content) => {
            inner_text_view(undo_stack, Text { content, lang: RwSignal::new(todo!("lang")) }, focus_element, id)
            .into_any()
        }
        InnerBlock::Lacuna(content, reason) => {
            let (old_content, set_old_content) = signal(content.get_untracked());
            let (old_reason, set_old_reason) = signal(reason.get_untracked());
            view! {
                <div>
                    <span
                        class="font-light text-xs">
                            "Lacuna because of "
                    </span>
                    <input
                    prop:value=reason
                    class="text-sm"
                    placeholder="reason"
                    autocomplete="false"
                    spellcheck="false"
                    on:input:target=move |ev| {
                        reason.set(ev.target().value());
                    }
                    on:change:target=move |ev| {
                        let current_old_reason = old_reason.get();
                        let new_reason = ev.target().value();
                        set_old_reason.set(new_reason.clone());
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Lacuna(Lacuna {
                                    reason: current_old_reason,
                                    n: todo!("lacuna-len"),
                                    unit: todo!("lacuna-unit"),
                                    cert: todo!("lacuna-cert"),
                                    content: Some(content.get_untracked()),
                                }),
                                Block::Lacuna(Lacuna {
                                    reason: new_reason,
                                    n: todo!("lacuna-len"),
                                    unit: todo!("lacuna-unit"),
                                    cert: todo!("lacuna-cert"),
                                    content: Some(content.get_untracked()),
                                })));
                    }/>
                    <span
                        class="font-light text-xs">
                        :
                    </span>
                    <br/>
                    <textarea
                    class="bg-orange-100 text-black font-mono"
                    id={format!("block-input-{id}")}
                    node_ref=focus_element
                    prop:value=content
                    autocomplete="false"
                    spellcheck="false"
                    rows=TEXTAREA_DEFAULT_ROWS
                    cols=TEXTAREA_DEFAULT_COLS
                    on:input:target=move |ev| {
                        content.set(ev.target().value());
                    }
                    on:change:target=move |ev| {
                        let current_old_content = old_content.get();
                        let new_content = ev.target().value();
                        set_old_content.set(new_content.clone());
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Lacuna(Lacuna {
                                    reason: reason.get_untracked(),
                                    n: todo!("lacuna-len"),
                                    unit: todo!("lacuna-unit"),
                                    cert: todo!("lacuna-cert"),
                                    content: Some(current_old_content),
                                }),
                                Block::Lacuna(Lacuna {
                                    reason: reason.get_untracked(),
                                    n: todo!("lacuna-len"),
                                    unit: todo!("lacuna-unit"),
                                    cert: todo!("lacuna-cert"),
                                    content: Some(new_content),
                                })));
                    }
                />
                </div>
            }
            .into_any()
        }
        InnerBlock::Uncertain(content, reason) => {
            let (old_content, set_old_content) = signal(content.get_untracked());
            let (old_reason, set_old_reason) = signal(reason.get_untracked());
            view! {
                <div>
                    <span
                        class="font-light text-xs">
                        "Uncertain because of "
                    </span>
                    <input
                    class="text-sm"
                    placeholder="reason"
                    autocomplete="false"
                    spellcheck="false"
                    prop:value=reason
                    on:input:target=move |ev| {
                        reason.set(ev.target().value());
                    }
                    on:change:target=move |ev| {
                        let current_old_reason = old_reason.get();
                        let new_reason = ev.target().value();
                        set_old_reason.set(new_reason.clone());
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Uncertain(Uncertain {
                                    lang: todo!("lang"),
                                    cert: todo!("cert"),
                                    agent: current_old_reason,
                                    content: content.get_untracked() }),
                                Block::Uncertain(Uncertain {
                                    lang: todo!("lang"),
                                    cert: todo!("cert"),
                                    agent: new_reason,
                                    content: content.get_untracked() }) ));
                    }/>
                    <span class="font-light text-xs">
                        :
                    </span>
                    <br/>
                    <textarea
                    id={format!("block-input-{id}")}
                    class="bg-orange-100 text-black font-mono"
                    node_ref=focus_element
                    autocomplete="false"
                    spellcheck="false"
                    rows=TEXTAREA_DEFAULT_ROWS
                    cols=TEXTAREA_DEFAULT_COLS
                    prop:value=content
                    on:input:target=move |ev| {
                        content.set(ev.target().value());
                    }
                    on:change:target=move |ev| {
                        let current_old_content = old_content.get();
                        let new_content = ev.target().value();
                        set_old_content.set(new_content.clone());
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Uncertain(Uncertain {
                                    lang: todo!("lang"),
                                    cert: todo!("cert"),
                                    agent: reason.get_untracked(),
                                    content: current_old_content }),
                                Block::Uncertain(Uncertain {
                                    lang: todo!("lang"),
                                    cert: todo!("cert"),
                                    agent: reason.get_untracked(),
                                    content: new_content }) ));
                    }
                />
                </div>
            }
            .into_any()
        }
        InnerBlock::Break(reason) => {
            let (old_reason, set_old_reason) = signal(reason.get_untracked());
            view! {
                    <div>
                        <p
                            class="font-light text-xs">
                            "Break: "
                        </p>
                        <select
                        id={format!("block-input-{id}")}
                        prop:value=reason.get().name()
                        on:input:target=move |ev| {
                            reason.set(ev.target().value().parse().expect("Only correct Names in the options for this select field."));
                        }
                        on:change:target=move |ev| {
                            let current_old_reason = old_reason.get();
                            let new_reason: BreakType = ev.target().value().parse().expect("Only correct Names in the options for this select field.");
                            set_old_reason.set(new_reason.clone());
                            undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                    Block::Break(current_old_reason),
                                    Block::Break(new_reason)));
                        }
                    >
                        <option value="Line">Line</option>
                        <option value="Column">Column</option>
                    </select>
                    </div>
                }.into_any()
        }
    }
}

impl EditorBlock {
    // construct a block with id, type, lang, content, and focus state
    pub fn new(
        id: i32,
        block_type: BlockType,
        lang: String,
        content: String,
        focus_on_load: bool,
    ) -> Self {
        Self {
            id,
            inner: InnerBlock::from_type_lang_and_content(block_type, lang, content),
            focus_on_load,
        }
    }

    /// Get this blocks id
    pub fn id(&self) -> i32 {
        self.id
    }

    /// Display this block
    pub(super) fn view(self) -> impl IntoView {
        view! {
            <span>
                // we probably do not want to show the blocks ID to the user
                // {self.id}
                // ":"
                <InnerView inner=self.inner id=self.id focus_on_load=self.focus_on_load></InnerView></span>
        }
    }

    pub(super) fn set_autoload(&mut self, focus_on_load: bool) {
        self.focus_on_load = focus_on_load
    }

    /// Overwrite the inner block with `new_inner` if it is currently `old_inner`
    ///
    /// Will clone new_inner if required, but not if the assert failed
    pub(super) fn overwrite_inner(&mut self, old_inner: &Block, new_inner: &Block) -> Option<()> {
        if *old_inner != self.inner {
            None
        } else {
            self.inner.overwrite_with(new_inner.clone());
            Some(())
        }
    }

    /// Split this block, returning new blocks and the index of the block which defaults as the
    /// newly inserted one.
    pub(super) fn split_at_selection(
        &self,
        start: usize,
        end: usize,
        new_block_type: BlockType,
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
/// TODO: rework this to compare all the things
impl PartialEq<Block> for InnerBlock {
    fn eq(&self, other: &Block) -> bool {
        match self {
            Self::Text(x) => match other {
                Block::Text(y) => y.content == x.get_untracked(),
                _ => false,
            },
            Self::Break(x) => match other {
                Block::Break(y) => *y == x.get_untracked(),
                _ => false,
            },
            Self::Lacuna(x, y) => match other {
                Block::Lacuna(l) => {
                    l.reason == y.get_untracked()
                        && if let Some(content) = &l.content {
                            *content == x.get_untracked()
                        } else {
                            x.get_untracked() == ""
                        }
                }
                _ => false,
            },
            Self::Uncertain(x, y) => match other {
                Block::Uncertain(u) => {
                    u.agent == y.get_untracked() && u.content == x.get_untracked()
                }
                _ => false,
            },
        }
    }
}
impl PartialEq<InnerBlock> for Block {
    fn eq(&self, other: &InnerBlock) -> bool {
        other.eq(self)
    }
}

/// Block type with data
///
/// NOTE: this could also be done with Traits and generic functions.
/// That would be nicer in a sense, but we are compiling into WASM, so binary size is more
/// important then nice generics imho. I keep it as this enum with some runtimechecks.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
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
    Break(RwSignal<BreakType>),
}
impl InnerBlock {
    /// overwrite own data with that given from new_block, but only if the types are the same
    fn overwrite_with(&mut self, new_block: Block) {
        match self {
            Self::Text(x) => match new_block {
                Block::Text(y) => {
                    *x.write() = y.content;
                }
                _ => {}
            },
            Self::Break(x) => match new_block {
                Block::Break(y) => {
                    *x.write() = y;
                }
                _ => {}
            },
            Self::Lacuna(x, y) => match new_block {
                Block::Lacuna(new_lacuna) => {
                    *x.write() = todo!("add text to gap");
                    *y.write() = new_lacuna.reason;
                }
                _ => {}
            },
            Self::Uncertain(x, y) => match new_block {
                Block::Uncertain(new_uncertain) => {
                    *x.write() = new_uncertain.content;
                    *y.write() = new_uncertain.agent;
                }
                _ => {}
            },
        }
    }

    /// Copy the metadata from [`self`] but get the content from another string
    pub fn clone_with_new_content(&self, content: String) -> Self {
        match self {
            Self::Text(_) => InnerBlock::Text(RwSignal::new(content.to_owned())),
            Self::Uncertain(_, y) => InnerBlock::Uncertain(RwSignal::new(content.to_owned()), *y),
            Self::Lacuna(_, y) => InnerBlock::Lacuna(RwSignal::new(content.to_owned()), *y),
            Self::Break(y) => InnerBlock::Break(*y),
        }
    }

    /// get this blocks content if this blocktype has content
    ///
    /// This is one of the functions which would be nicer with Traits, but here we need to return
    /// Option instead.
    pub fn content(&self) -> Option<guards::ReadGuard<String, guards::Plain<String>>> {
        match &self {
            Self::Text(el) => Some(el.read()),
            Self::Uncertain(el, _) => Some(el.read()),
            Self::Lacuna(el, _) => Some(el.read()),
            Self::Break(_) => None,
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
        new_block_type: BlockType,
    ) -> Vec<(InnerBlock, bool)> {
        let complete_value = match self.content() {
            Some(x) => x,
            // Block types without content can never fire split_at_selection,
            // so the function should return itself
            None => {
                return vec![(self.clone(), false)];
            }
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
            res.push((self.clone_with_new_content(content.to_owned()), false));
        };
        res.push((
            InnerBlock::from_type_lang_and_content(
                new_block_type,
                todo!("lang"),
                new_part.to_owned(),
            ),
            // we do want to autofocus on the middle block
            true,
        ));
        if let Some(content) = after_part {
            res.push((self.clone_with_new_content(content.to_owned()), false));
        };
        return res;
    }
}
impl FromTypeLangAndContent for InnerBlock {
    /// Create a new Block with content
    fn from_type_lang_and_content(block_type: BlockType, lang: String, content: String) -> Self {
        Block::from_type_lang_and_content(block_type, lang, content).into()
    }
}

/// Dehydrate [`InnerBlock`]
impl From<InnerBlock> for Block {
    fn from(value: InnerBlock) -> Self {
        match value {
            InnerBlock::Text(x) => Block::Text(Paragraph {
                lang: todo!(),
                content: x.get_untracked(),
            }),
            InnerBlock::Break(x) => Block::Break(x.get_untracked()),
            InnerBlock::Lacuna(x, y) => {
                todo!();
                Block::Lacuna(Lacuna {
                    reason: y.get_untracked(),
                    unit: todo!(),
                    n: todo!(),
                    cert: todo!(),
                    content: Some(x.get_untracked()),
                })
            }
            InnerBlock::Uncertain(x, y) => Block::Uncertain(Uncertain {
                lang: todo!(),
                cert: todo!(),
                agent: y.get_untracked(),
                content: x.get_untracked(),
            }),
        }
    }
}
/// Hydrate [`InnerBlockDry`]
impl From<Block> for InnerBlock {
    fn from(value: Block) -> Self {
        match value {
            Block::Uncertain(x) => {
                InnerBlock::Uncertain(RwSignal::new(x.content), RwSignal::new(x.lang))
            }
            Block::Text(x) => InnerBlock::Text(RwSignal::new(x.content)),
            Block::Break(x) => InnerBlock::Break(RwSignal::new(x)),
            Block::Lacuna(x) => InnerBlock::Lacuna(RwSignal::new(todo!()), RwSignal::new(x.reason)),
            Block::Anchor(x) => {
                todo!()
            }
            Block::Correction(x) => {
                todo!()
            }
            Block::Abbreviation(x) => {
                todo!()
            }
        }
    }
}

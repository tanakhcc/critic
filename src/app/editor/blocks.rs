//! The Types and associated functions for Blocks
//!
//! This module defines, what blocks are available, do and look like. Interaction with other
//! elements is handled in [`editor`](crate::app::editor) itself.

use critic_format::streamed::{
    Abbreviation, Anchor, Block, BlockType, BreakType, Correction, FromTypeLangAndContent, Lacuna, Paragraph, Uncertain, Version
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

fn inner_text_view(undo_stack: RwSignal<UnReStack>, paragraph: RwSignal::<Paragraph>, focus_element: leptos::prelude::NodeRef<Textarea>, id: i32) -> impl IntoView {
            // initialize the old content with the current one
            let initial_state = paragraph.get_untracked();
            let (old_content, set_old_content) = signal(initial_state.content);
            let (old_lang, set_old_lang) = signal(initial_state.lang);
            view! {
                <div>
                    <p
                        class="font-light text-xs">
                        "Raw Text: "
                    </p>
                    <input
                    prop:value=paragraph.with(|p| p.content.clone())
                    class="text-sm"
                    placeholder="reason"
                    autocomplete="false"
                    spellcheck="false"
                    on:input:target=move |ev| {
                        paragraph.update(|p| p.lang = ev.target().value())
                    }
                    on:change:target=move |ev| {
                        let current_old_lang = old_lang.get();
                        let new_lang = ev.target().value();
                        set_old_lang.set(new_lang.clone());
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Text(Paragraph { lang: current_old_lang, content: paragraph.read_untracked().content.clone() }),
                                Block::Text(Paragraph { lang: new_lang, content: paragraph.read_untracked().content.clone() }),
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
                    prop:value=paragraph.get_untracked().content
                    on:input:target=move |ev| {
                        //change the current content when updated
                        paragraph.write().content=ev.target().value();
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
                                    lang: paragraph.read().lang.clone(),
                                    content: current_old_content }),
                                Block::Text(Paragraph {
                                    lang: paragraph.read().lang.clone(),
                                    content: new_content }
                            )));
                    }
                />
                </div>
            }
}

fn inner_lacuna_view(undo_stack: RwSignal<UnReStack>, lacuna: RwSignal::<Lacuna>, focus_element: leptos::prelude::NodeRef<Textarea>, id: i32) -> impl IntoView {
    // clone the lacuna into a new block with separate tracking
    // `lacuna` itself will contain the displayed setting, `current_lacuna`
    // will contain the value from the last savepoint (of the Undo-stack)
    let current_lacuna = RwSignal::new(lacuna.get_untracked());
    view! {
        <div>
            <span
                class="font-light text-xs">
                    "Lacuna because of "
            </span>
            <input
            prop:value=lacuna.get_untracked().content
            class="text-sm"
            placeholder="reason"
            autocomplete="false"
            spellcheck="false"
            on:input:target=move |ev| {
                lacuna.write().reason = ev.target().value();
            }
            on:change:target=move |ev| {
                lacuna.write().reason = ev.target().value();
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Lacuna(current_lacuna.get_untracked()),
                        Block::Lacuna(lacuna.get_untracked().into()))
                    );
                // now set the new savepoint
                *current_lacuna.write() = lacuna.get_untracked();
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
            prop:value=lacuna.get_untracked().content
            autocomplete="false"
            spellcheck="false"
            rows=TEXTAREA_DEFAULT_ROWS
            cols=TEXTAREA_DEFAULT_COLS
            on:input:target=move |ev| {
                let x = ev.target().value();
                lacuna.write().content = if x.is_empty() {
                    None
                } else {
                    Some(x)
                };
            }
            on:change:target=move |ev| {
                let x = ev.target().value();
                lacuna.write().content = if x.is_empty() {
                    None
                } else {
                    Some(x)
                };
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Lacuna(current_lacuna.get_untracked()),
                        Block::Lacuna(lacuna.get_untracked().into()))
                    );
                // now set the new savepoint
                *current_lacuna.write() = lacuna.get_untracked();
            }
        />
        </div>
    }
}

fn inner_uncertain_view(undo_stack: RwSignal<UnReStack>, uncertain: RwSignal::<Uncertain>, focus_element: leptos::prelude::NodeRef<Textarea>, id: i32) -> impl IntoView {
    // clone the uncertain passage into a new block with separate tracking
    // `uncertain` itself will contain the displayed setting, `current_unceretain`
    // will contain the value from the last savepoint (of the Undo-stack)
    let current_uncertain = RwSignal::new(uncertain.get_untracked());
    view! {
        <div>
            <span
                class="font-light text-xs">
                    "Uncertain because of "
            </span>
            <input
            prop:value=uncertain.get_untracked().content
            class="text-sm"
            placeholder="reason"
            autocomplete="false"
            spellcheck="false"
            on:input:target=move |ev| {
                uncertain.write().agent = ev.target().value();
            }
            on:change:target=move |ev| {
                uncertain.write().agent = ev.target().value();
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Uncertain(current_uncertain.get_untracked()),
                        Block::Uncertain(uncertain.get_untracked().into()))
                    );
                // now set the new savepoint
                *current_uncertain.write() = uncertain.get_untracked();
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
            prop:value=uncertain.get_untracked().content
            autocomplete="false"
            spellcheck="false"
            rows=TEXTAREA_DEFAULT_ROWS
            cols=TEXTAREA_DEFAULT_COLS
            on:input:target=move |ev| {
                uncertain.write().content = ev.target().value();
            }
            on:change:target=move |ev| {
                uncertain.write().content = ev.target().value();
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Uncertain(current_uncertain.get_untracked()),
                        Block::Uncertain(uncertain.get_untracked().into()))
                    );
                // now set the new savepoint
                *current_uncertain.write() = uncertain.get_untracked();
            }
        />
        </div>
    }
}

fn inner_break_view(undo_stack: RwSignal<UnReStack>, break_block: RwSignal::<BreakType>, focus_element: leptos::prelude::NodeRef<Textarea>, id: i32) -> impl IntoView {
    let current_break_block = RwSignal::new(break_block.get_untracked());
    view! {
            <div>
                <p
                    class="font-light text-xs">
                    "Break: "
                </p>
                <select
                id={format!("block-input-{id}")}
                prop:value=break_block.get_untracked().name()
                on:input:target=move |ev| {
                    *break_block.write() = ev.target().value().parse().expect("Only correct Names in the options for this select field.");
                }
                on:change:target=move |ev| {
                    *break_block.write() = ev.target().value().parse().expect("Only correct Names in the options for this select field.");
                    undo_stack.write().push_undo(UnReStep::new_data_change(id,
                            Block::Break(current_break_block.get_untracked()),
                            Block::Break(break_block.get_untracked())));
                    *current_break_block.write() = break_block.get_untracked();
                }
            >
                <option value="Line">Line</option>
                <option value="Column">Column</option>
            </select>
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
        InnerBlock::Text(paragraph) => {
            inner_text_view(undo_stack, paragraph, focus_element, id)
            .into_any()
        }
        InnerBlock::Lacuna(lacuna) => {
            inner_lacuna_view(undo_stack, lacuna, focus_element, id)
            .into_any()
        }
        InnerBlock::Uncertain(uncertain) => {
            inner_uncertain_view(undo_stack, uncertain, focus_element, id)
            .into_any()
        }
        InnerBlock::Break(break_block) => {
            inner_break_view(undo_stack, break_block, focus_element, id)
            .into_any()
        }
        _ => {
            // Add Anchor, Correction, Abbreviation etc.
            todo!()
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
    /// and only if the types match.
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
impl PartialEq<Block> for InnerBlock {
    fn eq(&self, other: &Block) -> bool {
        match self {
            InnerBlock::Text(x) => {
                match other {
                    Block::Text(y) => x.read() == *y,
                    _ => false,
                }
            }
            InnerBlock::Break(x) => {
                match other {
                    Block::Break(y) => x.read() == *y,
                    _ => false,
                }
            }
            InnerBlock::Lacuna(x) => {
                match other {
                    Block::Lacuna(y) => x.read() == *y,
                    _ => false,
                }
            }
            InnerBlock::Anchor(x) => {
                match other {
                    Block::Anchor(y) => x.read() == *y,
                    _ => false,
                }
            }
            InnerBlock::Correction(x) => {
                match other {
                    Block::Correction(y) => x.read() == *y,
                    _ => false,
                }
            }
            InnerBlock::Uncertain(x) => {
                match other {
                    Block::Uncertain(y) => x.read() == *y,
                    _ => false,
                }
            }
            InnerBlock::Abbreviation(x) => {
                match other {
                    Block::Abbreviation(y) => x.read() == *y,
                    _ => false,
                }
            }
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
/// This is the same as [`critic_format::streamed::Block`], but all data is wrapped in an [`RwSignal`]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub(super) enum InnerBlock {
    /// A break in the text - line or column break
    Break(RwSignal<BreakType>),
    /// A lacuna in the manuscript
    Lacuna(RwSignal<Lacuna>),
    /// An anchor - the beginning of a verse
    Anchor(RwSignal<Anchor>),
    // Normal unmarked text
    Text(RwSignal<Paragraph>),
    /// A correction in the manuscript - where one scribal hand has overwritte / struck through / .. a text that was present earlier
    Correction(RwSignal<Correction>),
    // A part of text that is damaged but still legible
    Uncertain(RwSignal<Uncertain>),
    // An expanded abbreviation
    Abbreviation(RwSignal<Abbreviation>),
}
impl InnerBlock {
    /// overwrite own data with that given from new_block, but only if the types are the same
    fn overwrite_with(&mut self, new_block: Block) {
        match self {
            Self::Text(x) => match new_block {
                Block::Text(y) => {
                    *x.write() = y;
                }
                _ => {}
            },
            Self::Break(x) => match new_block {
                Block::Break(y) => {
                    *x.write() = y;
                }
                _ => {}
            },
            Self::Lacuna(x) => match new_block {
                Block::Lacuna(new_lacuna) => {
                    *x.write() = new_lacuna;
                }
                _ => {}
            },
            Self::Uncertain(x) => match new_block {
                Block::Uncertain(new_uncertain) => {
                    *x.write() = new_uncertain;
                }
                _ => {}
            },
            Self::Anchor(x) => match new_block {
                Block::Anchor(new_anchor) => {
                    *x.write() = new_anchor;
                }
                _ => {}
            }
            Self::Correction(x) => match new_block {
                Block::Correction(new_correction) => {
                    *x.write() = new_correction;
                }
                _ => {}
            }
            Self::Abbreviation(x) => match new_block {
                Block::Abbreviation(new_abbreviation) => {
                    *x.write() = new_abbreviation
                }
                _ => {}
            }
        }
    }

    fn clone_with_new_content(&self, new_content: String) -> InnerBlock {
        match self {
            InnerBlock::Break(_) => self.clone(),
            InnerBlock::Lacuna(lacuna) => {
                InnerBlock::Lacuna(RwSignal::new(Lacuna {
                    lang: lacuna.read().lang.clone(),
                    cert: lacuna.read().cert.clone(),
                    content: Some(new_content),
                    n: lacuna.read().n,
                    unit: lacuna.read().unit.clone(),
                    reason: lacuna.read().reason.clone(),
                }))
            }
            InnerBlock::Anchor(_) => self.clone(),
            InnerBlock::Text(paragraph) => {
                InnerBlock::Text(RwSignal::new(Paragraph { lang: paragraph.read().lang.clone(), content: new_content }))
            }
            InnerBlock::Correction(correction) => {
                InnerBlock::Correction(RwSignal::new(Correction { lang: correction.read().lang.clone(), versions: vec![Version { lang: correction.read().lang.clone(), hand: None, content: new_content }] }))
            }
            InnerBlock::Uncertain(uncertain) => {
                InnerBlock::Uncertain(RwSignal::new(Uncertain { lang: uncertain.read().lang.clone(), cert: uncertain.read().cert.clone(), agent: uncertain.read().agent.clone(), content: new_content }))
            }
            InnerBlock::Abbreviation(abbreviation) => {
                InnerBlock::Abbreviation(RwSignal::new(Abbreviation { lang: abbreviation.read().lang.clone(), surface: new_content.clone(), expansion: new_content }))
            }
        }
    }

    /// The primary surface content of this block
    ///
    /// i.e. the most natural reconstruction of what is physically on the MS
    fn content(&self) -> Option<String> {
        match self {
            InnerBlock::Text(x) => {
                Some(x.read().content.to_string())
            }
            InnerBlock::Break(_) => None,
            InnerBlock::Lacuna(x) => {
                x.read().content.clone()
            }
            InnerBlock::Anchor(_) => None,
            InnerBlock::Uncertain(x) => {
                Some(x.read().content.to_string())
            }
            InnerBlock::Correction(x) => {
                x.read().versions.first().map(|v| v.content.to_string())
            }
            InnerBlock::Abbreviation(x) => {
                Some(x.read().surface.to_string())
            }
        }
    }

    /// The primary language for this block if applicable
    fn lang(&self) -> Option<String> {
        match self {
            InnerBlock::Text(x) => {
                Some(x.read().lang.clone())
            }
            InnerBlock::Break(_) => None,
            InnerBlock::Lacuna(x) => {
                x.read().lang.clone()
            }
            InnerBlock::Anchor(_) => None,
            InnerBlock::Uncertain(x) => {
                Some(x.read().lang.clone())
            }
            InnerBlock::Correction(x) => {
                Some(x.read().lang.clone())
            }
            InnerBlock::Abbreviation(x) => {
                Some(x.read().lang.clone())
            }
        }
    }

    /// Split this block into 1-3 new blocks, so that the content in [start, end] is a new block
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
            // so the function should panic
            None => {
                unreachable!("split_at_selection cannot be called when the Blocktype has no content.");
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
                self.lang().unwrap_or_else(|| "".to_string()),
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
            InnerBlock::Text(x) => {
                Block::Text(x.get_untracked())
            }
            InnerBlock::Break(x) => {
                Block::Break(x.get_untracked())
            },
            InnerBlock::Lacuna(x) => {
                Block::Lacuna(x.get_untracked())
            }
            InnerBlock::Anchor(x) => {
                Block::Anchor(x.get_untracked())
            },
            InnerBlock::Uncertain(x) => {
                Block::Uncertain(x.get_untracked())
            }
            InnerBlock::Correction(x) => {
                Block::Correction(x.get_untracked())
            }
            InnerBlock::Abbreviation(x) => {
                Block::Abbreviation(x.get_untracked())
            }
        }
    }
}
/// Hydrate [`InnerBlockDry`]
impl From<Block> for InnerBlock {
    fn from(value: Block) -> Self {
        match value {
            Block::Text(x) => {
                InnerBlock::Text(RwSignal::new(x))
            }
            Block::Break(x) => {
                InnerBlock::Break(RwSignal::new(x))
            },
            Block::Lacuna(x) => {
                InnerBlock::Lacuna(RwSignal::new(x))
            }
            Block::Anchor(x) => {
                InnerBlock::Anchor(RwSignal::new(x))
            },
            Block::Uncertain(x) => {
                InnerBlock::Uncertain(RwSignal::new(x))
            }
            Block::Correction(x) => {
                InnerBlock::Correction(RwSignal::new(x))
            }
            Block::Abbreviation(x) => {
                InnerBlock::Abbreviation(RwSignal::new(x))
            }
        }
    }
}

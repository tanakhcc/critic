//! The Types and associated functions for Blocks
//!
//! This module defines, what blocks are available, do and look like. Interaction with other
//! elements is handled in [`editor`](crate::app::editor) itself.

use critic_format::streamed::{
    Abbreviation, Anchor, Block, BlockType, BreakType, Correction, FromTypeLangAndContent, Lacuna,
    Paragraph, Uncertain, Version,
};
use leptos::{html::Textarea, prelude::*};
use serde::{Deserialize, Serialize};

use super::{UnReStack, UnReStep};

use crate::app::accordion::{Accordion, Align, Item, List};

const TEXTAREA_DEFAULT_ROWS: i32 = 2;
const TEXTAREA_DEFAULT_COLS: i32 = 30;

/// A single block that we change in the editor
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub(super) struct EditorBlock {
    /// ID of the block (i.e. creation-order, NOT position)
    pub id: usize,
    /// The actual content
    pub inner: InnerBlock,
    /// Should this block focus when loaded?
    pub focus_on_load: bool,
}

#[component]
fn CogSvg() -> impl IntoView {
    view!{
<svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6"><path stroke-linecap="round" stroke-linejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.325.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 0 1 1.37.49l1.296 2.247a1.125 1.125 0 0 1-.26 1.431l-1.003.827c-.293.241-.438.613-.43.992a7.723 7.723 0 0 1 0 .255c-.008.378.137.75.43.991l1.004.827c.424.35.534.955.26 1.43l-1.298 2.247a1.125 1.125 0 0 1-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.47 6.47 0 0 1-.22.128c-.331.183-.581.495-.644.869l-.213 1.281c-.09.543-.56.94-1.11.94h-2.594c-.55 0-1.019-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 0 1-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 0 1-1.369-.49l-1.297-2.247a1.125 1.125 0 0 1 .26-1.431l1.004-.827c.292-.24.437-.613.43-.991a6.932 6.932 0 0 1 0-.255c.007-.38-.138-.751-.43-.992l-1.004-.827a1.125 1.125 0 0 1-.26-1.43l1.297-2.247a1.125 1.125 0 0 1 1.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.086.22-.128.332-.183.582-.495.644-.869l.214-1.28Z" /><path stroke-linecap="round" stroke-linejoin="round" d="M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z" /></svg>
    }
}

fn inner_text_view(
    undo_stack: RwSignal<UnReStack>,
    paragraph: RwSignal<Paragraph>,
    focus_element: leptos::prelude::NodeRef<Textarea>,
    id: usize,
) -> impl IntoView {
    let current_paragraph = RwSignal::new(paragraph.get_untracked());
    let config_expanded = signal(false);
    view! {
        <div class="flex justify-between">
        <div>
            <p
                class="font-light text-xs">
                "Raw Text: "
            </p>
            <textarea
            class="bg-yellow-100 text-black font-mono"
            id={format!("block-input-{id}")}
            node_ref=focus_element
            autocomplete="false"
            spellcheck="false"
            rows=TEXTAREA_DEFAULT_ROWS
            cols=TEXTAREA_DEFAULT_COLS
            // reactive, so undo/redo actions can change the view
            prop:value=move || paragraph.read().content.clone()
            on:input:target=move |ev| {
                //change the current content when updated
                paragraph.write().content=ev.target().value();
            }
            on:change:target=move |ev| {
                paragraph.write().content = ev.target().value();
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Text(current_paragraph.get_untracked()),
                        Block::Text(paragraph.get_untracked().into()))
                    );
                // now set the new savepoint
                current_paragraph.write().content = paragraph.read_untracked().content.clone();
            }
        />
        </div>
        <Accordion
            expand={config_expanded}
            expanded={Box::new(|| view! { <CogSvg/> }.into_any())}
            collapsed={Box::new(|| view! { <CogSvg/> }.into_any())}
        >
            <List>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Language: "</span>
                    <input
                    prop:value=move || paragraph.read().lang.clone()
                    class="text-sm"
                    placeholder="language"
                    autocomplete="false"
                    spellcheck="false"
                    on:input:target=move |ev| {
                        paragraph.write().lang = ev.target().value();
                    }
                    on:change:target=move |ev| {
                        paragraph.write().lang = ev.target().value();
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Text(current_paragraph.get_untracked()),
                                Block::Text(paragraph.get_untracked().into()))
                            );
                        // now set the new savepoint
                        current_paragraph.write().lang = paragraph.read_untracked().lang.clone();
                    }/>
                </Item>
            </List>
        </Accordion>
        </div>
    }
}

fn inner_lacuna_view(
    undo_stack: RwSignal<UnReStack>,
    lacuna: RwSignal<Lacuna>,
    id: usize,
) -> impl IntoView {
    // clone the lacuna into a new block with separate tracking
    // `lacuna` itself will contain the displayed setting, `current_lacuna`
    // will contain the value from the last savepoint (of the Undo-stack)
    let current_lacuna = RwSignal::new(lacuna.get_untracked());
    let config_expanded = signal(false);
    view! {
        <div class="flex justify-between">
        <div>
            <span
                class="font-light text-xs">
                    "Lacuna because of "
            </span>
            <input
            prop:value=move || lacuna.read().reason.clone()
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
                current_lacuna.write().reason = lacuna.read_untracked().reason.clone();
            }/>
        </div>
        <Accordion
            expand={config_expanded}
            expanded={Box::new(|| view! { <CogSvg/> }.into_any())}
            collapsed={Box::new(|| view! { <CogSvg/> }.into_any())}
        >
            <List>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Extent: "</span>
                    <input
                    prop:value=move || lacuna.read().n.clone()
                    class="text-sm"
                    placeholder="n"
                    autocomplete="false"
                    spellcheck="false"
                    on:input:target=move |ev| {
                        // here we can allow the value to be unparsable, but we want to prevent
                        // this if possible
                        let x = ev.target().value();
                        if x.is_empty() {
                        } else {
                            lacuna.write().n = ev.target().value().parse().unwrap_or_else(|_| 1);
                        }
                    }
                    on:change:target=move |ev| {
                        // just throw away values that are not parsable
                        lacuna.write().n = ev.target().value().parse().unwrap_or_else(|_| 1);
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Lacuna(current_lacuna.get_untracked()),
                                Block::Lacuna(lacuna.get_untracked().into()))
                            );
                        // now set the new savepoint
                        current_lacuna.write().n = lacuna.get_untracked().n;
                    }/>
                </Item>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Unit of Extent: "</span>
                    <select
                    prop:value=move || lacuna.read().unit.name()
                    on:input:target=move |ev| {
                        lacuna.write().unit = ev.target().value().parse().expect("Only correct Names in the options for this select field.");
                    }
                    on:change:target=move |ev| {
                        lacuna.write().unit = ev.target().value().parse().expect("Only correct Names in the options for this select field.");
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Lacuna(current_lacuna.get_untracked()),
                                Block::Lacuna(lacuna.get_untracked())));
                        current_lacuna.write().unit = lacuna.get_untracked().unit;
                    }
                >
                    <option value="Character">Column</option>
                    <option value="Line">Line</option>
                    <option value="Column">Column</option>
                </select>
                </Item>
            </List>
        </Accordion>
        </div>
    }
}

fn inner_uncertain_view(
    undo_stack: RwSignal<UnReStack>,
    uncertain: RwSignal<Uncertain>,
    focus_element: leptos::prelude::NodeRef<Textarea>,
    id: usize,
) -> impl IntoView {
    // clone the uncertain passage into a new block with separate tracking
    // `uncertain` itself will contain the displayed setting, `current_unceretain`
    // will contain the value from the last savepoint (of the Undo-stack)
    let current_uncertain = RwSignal::new(uncertain.get_untracked());

    let config_expanded = signal(false);
    view! {
        <div class="flex justify-between">
        <div>
            // header line
            <span
                class="font-light text-xs">
                    "Uncertain because of "
            </span>
            // reason for the uncertainty
            <input
            prop:value=move || uncertain.read().agent.clone()
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
                current_uncertain.write().agent = uncertain.get_untracked().agent;
            }/>
            <span
                class="font-light text-xs">
                :
            </span>
            <br/>
            // proposed (reconstructed) content
            <textarea
            class="bg-orange-100 text-black font-mono"
            id={format!("block-input-{id}")}
            node_ref=focus_element
            prop:value=move || uncertain.read().content.clone()
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
                current_uncertain.write().content = uncertain.get_untracked().content;
            }
        />
        </div>
        <Accordion
            expand={config_expanded}
            expanded={Box::new(|| view! { <CogSvg/> }.into_any())}
            collapsed={Box::new(|| view! { <CogSvg/> }.into_any())}
        >
            <List>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Language: "</span>
                    <input
                    prop:value=move || uncertain.read().lang.clone()
                    class="text-sm"
                    placeholder="language"
                    autocomplete="false"
                    spellcheck="false"
                    on:input:target=move |ev| {
                        uncertain.write().lang = ev.target().value();
                    }
                    on:change:target=move |ev| {
                        uncertain.write().lang = ev.target().value();
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Uncertain(current_uncertain.get_untracked()),
                                Block::Uncertain(uncertain.get_untracked().into()))
                            );
                        // now set the new savepoint
                        current_uncertain.write().lang = uncertain.read_untracked().lang.clone();
                    }/>
                </Item>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Certainty: "</span>
                    <input
                    // the unwrap_or_else is required, because cert can be None but we want to push
                    // "" to the user in that case
                    prop:value=move || uncertain.read().cert.clone().unwrap_or_else(String::default)
                    class="text-sm"
                    placeholder="certainty"
                    autocomplete="false"
                    spellcheck="false"
                    on:input:target=move |ev| {
                        let x = ev.target().value();
                        uncertain.write().cert = (!x.is_empty()).then(|| x);
                    }
                    on:change:target=move |ev| {
                        let x = ev.target().value();
                        uncertain.write().cert = (!x.is_empty()).then(|| x);
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Uncertain(current_uncertain.get_untracked()),
                                Block::Uncertain(uncertain.get_untracked().into()))
                            );
                        // now set the new savepoint
                        current_uncertain.write().cert = uncertain.read_untracked().cert.clone();
                    }/>
                </Item>
            </List>
        </Accordion>
        </div>
    }
}

fn inner_break_view(
    undo_stack: RwSignal<UnReStack>,
    break_block: RwSignal<BreakType>,
    id: usize,
) -> impl IntoView {
    let current_break_block = RwSignal::new(break_block.get_untracked());
    view! {
            <div>
                <p
                    class="font-light text-xs">
                    "Break: "
                </p>
                <select
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
fn InnerView(inner: InnerBlock, id: usize, focus_on_load: bool) -> impl IntoView {
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
            inner_text_view(undo_stack, paragraph, focus_element, id).into_any()
        }
        InnerBlock::Lacuna(lacuna) => {
            inner_lacuna_view(undo_stack, lacuna, id).into_any()
        }
        InnerBlock::Uncertain(uncertain) => {
            inner_uncertain_view(undo_stack, uncertain, focus_element, id).into_any()
        }
        InnerBlock::Break(break_block) => {
            inner_break_view(undo_stack, break_block, id).into_any()
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
        id: usize,
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
    pub fn id(&self) -> usize {
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
        new_index: &mut usize,
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
            InnerBlock::Text(x) => match other {
                Block::Text(y) => x.read_untracked() == *y,
                _ => false,
            },
            InnerBlock::Break(x) => match other {
                Block::Break(y) => x.read_untracked() == *y,
                _ => false,
            },
            InnerBlock::Lacuna(x) => match other {
                Block::Lacuna(y) => x.read_untracked() == *y,
                _ => false,
            },
            InnerBlock::Anchor(x) => match other {
                Block::Anchor(y) => x.read_untracked() == *y,
                _ => false,
            },
            InnerBlock::Correction(x) => match other {
                Block::Correction(y) => x.read_untracked() == *y,
                _ => false,
            },
            InnerBlock::Uncertain(x) => match other {
                Block::Uncertain(y) => x.read_untracked() == *y,
                _ => false,
            },
            InnerBlock::Abbreviation(x) => match other {
                Block::Abbreviation(y) => x.read_untracked() == *y,
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
/// This is the same as [`critic_format::streamed::Block`], but all data is wrapped in an [`RwSignal`]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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
            },
            Self::Correction(x) => match new_block {
                Block::Correction(new_correction) => {
                    *x.write() = new_correction;
                }
                _ => {}
            },
            Self::Abbreviation(x) => match new_block {
                Block::Abbreviation(new_abbreviation) => *x.write() = new_abbreviation,
                _ => {}
            },
        }
    }

    fn clone_with_new_content(&self, new_content: String) -> InnerBlock {
        match self {
            InnerBlock::Break(_) => self.clone(),
            InnerBlock::Lacuna(lacuna) => InnerBlock::Lacuna(RwSignal::new(Lacuna {
                cert: lacuna.read_untracked().cert.clone(),
                n: lacuna.read_untracked().n,
                unit: lacuna.read_untracked().unit.clone(),
                reason: lacuna.read_untracked().reason.clone(),
            })),
            InnerBlock::Anchor(_) => self.clone(),
            InnerBlock::Text(paragraph) => InnerBlock::Text(RwSignal::new(Paragraph {
                lang: paragraph.read_untracked().lang.clone(),
                content: new_content,
            })),
            InnerBlock::Correction(correction) => {
                InnerBlock::Correction(RwSignal::new(Correction {
                    lang: correction.read_untracked().lang.clone(),
                    versions: vec![Version {
                        lang: correction.read_untracked().lang.clone(),
                        hand: None,
                        content: new_content,
                    }],
                }))
            }
            InnerBlock::Uncertain(uncertain) => InnerBlock::Uncertain(RwSignal::new(Uncertain {
                lang: uncertain.read_untracked().lang.clone(),
                cert: uncertain.read_untracked().cert.clone(),
                agent: uncertain.read_untracked().agent.clone(),
                content: new_content,
            })),
            InnerBlock::Abbreviation(abbreviation) => {
                InnerBlock::Abbreviation(RwSignal::new(Abbreviation {
                    lang: abbreviation.read_untracked().lang.clone(),
                    surface: new_content.clone(),
                    expansion: new_content,
                }))
            }
        }
    }

    /// The primary surface content of this block
    ///
    /// i.e. the most natural reconstruction of what is physically on the MS
    fn content(&self) -> Option<String> {
        match self {
            InnerBlock::Text(x) => Some(x.read_untracked().content.to_string()),
            InnerBlock::Break(_) => None,
            InnerBlock::Lacuna(x) => None,
            InnerBlock::Anchor(_) => None,
            InnerBlock::Uncertain(x) => Some(x.read_untracked().content.to_string()),
            InnerBlock::Correction(x) => x
                .read_untracked()
                .versions
                .first()
                .map(|v| v.content.to_string()),
            InnerBlock::Abbreviation(x) => Some(x.read_untracked().surface.to_string()),
        }
    }

    /// The primary language for this block if applicable
    fn lang(&self) -> Option<String> {
        match self {
            InnerBlock::Text(x) => Some(x.read_untracked().lang.clone()),
            InnerBlock::Break(_) => None,
            InnerBlock::Lacuna(x) => None,
            InnerBlock::Anchor(_) => None,
            InnerBlock::Uncertain(x) => Some(x.read_untracked().lang.clone()),
            InnerBlock::Correction(x) => Some(x.read_untracked().lang.clone()),
            InnerBlock::Abbreviation(x) => Some(x.read_untracked().lang.clone()),
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
                unreachable!(
                    "split_at_selection cannot be called when the Blocktype has no content."
                );
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
            InnerBlock::Text(x) => Block::Text(x.get_untracked()),
            InnerBlock::Break(x) => Block::Break(x.get_untracked()),
            InnerBlock::Lacuna(x) => Block::Lacuna(x.get_untracked()),
            InnerBlock::Anchor(x) => Block::Anchor(x.get_untracked()),
            InnerBlock::Uncertain(x) => Block::Uncertain(x.get_untracked()),
            InnerBlock::Correction(x) => Block::Correction(x.get_untracked()),
            InnerBlock::Abbreviation(x) => Block::Abbreviation(x.get_untracked()),
        }
    }
}
/// Hydrate [`InnerBlockDry`]
impl From<Block> for InnerBlock {
    fn from(value: Block) -> Self {
        match value {
            Block::Text(x) => InnerBlock::Text(RwSignal::new(x)),
            Block::Break(x) => InnerBlock::Break(RwSignal::new(x)),
            Block::Lacuna(x) => InnerBlock::Lacuna(RwSignal::new(x)),
            Block::Anchor(x) => InnerBlock::Anchor(RwSignal::new(x)),
            Block::Uncertain(x) => InnerBlock::Uncertain(RwSignal::new(x)),
            Block::Correction(x) => InnerBlock::Correction(RwSignal::new(x)),
            Block::Abbreviation(x) => InnerBlock::Abbreviation(RwSignal::new(x)),
        }
    }
}

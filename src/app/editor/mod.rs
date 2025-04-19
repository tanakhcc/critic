//! Everything needed for the raw editor functionality is in this module.
//!
//! This is the GUI-area and directly related APIs/server functions to save its data.

use leptos::{
    ev::keydown,
    logging::log,
    prelude::*,
};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    hooks::use_params,
    params::Params,
    path, StaticSegment,
};
use leptos_use::{use_document, use_event_listener};
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct EditorBlock {
    id: i32,
    inner: InnerBlock,
    focus_on_load: bool,
}
impl EditorBlock {
    /// Split this block, returning new blocks and the index of the block which defaults as the
    /// newly inserted one.
    fn split_at_selection(
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
enum InnerBlockType {
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
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
enum InnerBlock {
    /// Raw text without special markup
    /// text
    Text(RwSignal<String>),
    /// A part of Text with uncertainty
    /// (proposed-text, reason)
    Uncertain(RwSignal<String>, RwSignal<String>),
    /// A part of Text that is absent or entirely unreadable
    /// (proposed-text, reason)
    Lacuna(RwSignal<String>, RwSignal<String>),
}
impl InnerBlock {
    fn new_from_type(block_type: InnerBlockType) -> InnerBlock {
        match block_type {
            InnerBlockType::Text => InnerBlock::Text(RwSignal::<String>::default()),
            InnerBlockType::Uncertain => {
                InnerBlock::Uncertain(RwSignal::<String>::default(), RwSignal::<String>::default())
            }
            InnerBlockType::Lacuna=> {
                InnerBlock::Lacuna(RwSignal::<String>::default(), RwSignal::<String>::default())
            }
        }
    }

    fn view(self, id: i32, do_focus: bool) -> impl IntoView {
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
            InnerBlock::Text(x) => {
                view! {
                    <div>
                        <p>"Raw Text: "</p>
                        <input node_ref=focus_element id={format!("block-input-{id}")} value=x.get() on:input:target=move |ev| {
                            x.set(ev.target().value());
                        }/>
                    </div>
                }.into_any()
            }
            InnerBlock::Uncertain(x, y) => {
                view! {
                    <div>
                        <p>"Uncertain: "</p>
                        <input id={format!("block-input-{id}")} value=x.get() on:input:target=move |ev| {
                            x.set(ev.target().value());
                        }/>
                        // we want to focus on the uncertainty for a new uncertain passage
                        // it is most likely that someone took a part of Text and marked a part as
                        // uncertain. In this case, the main content is already correct but the reasons
                        // needs to be supplied next
                        <input node_ref=focus_element value=y.get() on:input:target=move |ev| {
                            y.set(ev.target().value());
                        }/>
                    </div>
                }.into_any()
            }
            InnerBlock::Lacuna(x, y) => {
                view! {
                    <div>
                        <p>"Lacuna: "</p>
                        <input id={format!("block-input-{id}")} value=x.get() on:input:target=move |ev| {
                            x.set(ev.target().value());
                        }/>
                        // we want to focus on the reason for a new lacunous passage
                        // it is most likely that someone took a part of Text and marked a part as
                        // lacuna. In this case, the main content is already correct but the reasons
                        // needs to be supplied next
                        <input node_ref=focus_element value=y.get() on:input:target=move |ev| {
                            y.set(ev.target().value());
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
                    Self::Lacuna(_, y) => {
                        InnerBlock::Lacuna(RwSignal::new(content.to_owned()), *y)
                    }
                },
                false,
            ));
        };
        res.push((
            match new_block_type {
                InnerBlockType::Text => InnerBlock::Text(RwSignal::new(new_part.to_owned())),
                InnerBlockType::Uncertain => InnerBlock::Uncertain(
                    RwSignal::new(new_part.to_owned()),
                    RwSignal::<String>::default(),
                ),
                InnerBlockType::Lacuna => InnerBlock::Lacuna(
                    RwSignal::new(new_part.to_owned()),
                    RwSignal::<String>::default(),
                ),
            },
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
                    Self::Lacuna(_, y) => {
                        InnerBlock::Lacuna(RwSignal::new(content.to_owned()), *y)
                    }
                },
                false,
            ));
        };
        return res;
    }
}

fn new_node(
    physical_index_maybe: impl Fn(i32) -> Option<usize>,
    blocks: ReadSignal<Vec<EditorBlock>>,
    set_blocks: WriteSignal<Vec<EditorBlock>>,
    next_id: ReadSignal<i32>,
    set_next_id: WriteSignal<i32>,
    block_type: InnerBlockType,
) {
    let active_element = match use_document().active_element() {
        Some(el) => el,
        None => {
            return;
        }
    };
    let primary_input = match active_element.dyn_into::<HtmlInputElement>() {
        Ok(el) => el,
        Err(_) => {
            return;
        }
    };
    let current_select_start = primary_input.selection_start().unwrap_or(None);
    let current_select_end = primary_input.selection_end().unwrap_or(None);
    let complete_value = primary_input.value();
    // get the block index we are in right now
    let id_stripped = &primary_input.id()[12..];
    let id = match id_stripped.parse::<i32>() {
        Ok(el) => el,
        Err(_) => {
            return;
        }
    };
    // split this element in blocks
    if let Some(physical_index) = physical_index_maybe(id) {
        // this one block will usually be split in three: before-selection, selection,
        // after-selection
        match (current_select_start, current_select_end) {
            (Some(x), Some(y)) => {
                // convert indices to byte offsets (they are given as utf-8 character indices)
                let mut indices = complete_value.char_indices().map(|(i, _)| i);
                let start_utf8 = match indices.nth(x as usize) {
                    Some(el) => el,
                    None => complete_value.len(),
                };
                let end_utf8 = if x == y {
                    start_utf8
                } else {
                    indices
                        .nth((y - x - 1) as usize)
                        .unwrap_or(complete_value.len())
                };
                let new_blocks = match blocks.read().get(physical_index) {
                    Some(el) => el.split_at_selection(
                        start_utf8,
                        end_utf8,
                        block_type,
                        &mut set_next_id.write(),
                    ),
                    None => {
                        return;
                    }
                };
                // replace the block currently at physical_index with the new blocks
                set_blocks
                    .write()
                    .splice(physical_index..physical_index + 1, new_blocks);
            }
            _ => {
                // nothing selected, add a new empty node after this one
                set_blocks.write().insert(
                    physical_index,
                    EditorBlock {
                        id: next_id.get(),
                        inner: InnerBlock::new_from_type(block_type),
                        focus_on_load: true,
                    },
                );
                *set_next_id.write() += 1;
            }
        };
    };
}

#[component]
fn Editor() -> impl IntoView {
    let initial_id = 1;
    let (next_id, set_next_id) = signal(initial_id);
    let init_blocks = vec![];
    let (blocks, set_blocks) = signal(init_blocks);
    let add_block = move |_| {
        set_blocks.update(|bs| {
            bs.push(EditorBlock {
                id: next_id.get(),
                inner: InnerBlock::Text(RwSignal::new("some other text".to_owned())),
                // focus on new blocks
                focus_on_load: true,
            });
            set_next_id.update(|idx| *idx += 1);
        })
    };

    let physical_index_maybe = move |id: i32| blocks.read().iter().position(|b| b.id == id);

    let index_if_not_first = move |id: i32| {
        if let Some(physical_index) = physical_index_maybe(id) {
            if physical_index != 0 {
                Some(physical_index)
            } else {
                None
            }
        } else {
            None
        }
    };

    let move_up_button = move |id| {
        if let Some(physical_index) = index_if_not_first(id) {
            Some(view! {
                <button on:click=move |_| {
                    set_blocks.write().swap(physical_index, physical_index - 1);
                }>"Move this thingy up"</button>
            })
        } else {
            None
        }
    };

    let index_if_not_last = move |id: i32| {
        if let Some(physical_index) = physical_index_maybe(id) {
            if physical_index < blocks.read().len() - 1 {
                Some(physical_index)
            } else {
                None
            }
        } else {
            None
        }
    };

    let move_down_button = move |id| {
        if let Some(physical_index) = index_if_not_last(id) {
            Some(view! {
                <button on:click=move |_| {
                    set_blocks.write().swap(physical_index, physical_index + 1);
                }>"Move this thingy down"</button>
            })
        } else {
            None
        }
    };

    // the keyboard-shortcut listener
    let _cleanup = use_event_listener(use_document(), keydown, move |evt| {
        log!("{}", evt.key_code());
        // <ctrl>-<alt>-T (new Text)
        if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 84 {
            new_node(
                physical_index_maybe,
                blocks,
                set_blocks,
                next_id,
                set_next_id,
                InnerBlockType::Text,
            );
        // <ctrl>-<alt>-U (new Uncertain)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 85 {
            new_node(
                physical_index_maybe,
                blocks,
                set_blocks,
                next_id,
                set_next_id,
                InnerBlockType::Uncertain,
            )
        // <ctrl>-<alt>-L (new Lacuna)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 76 {
            new_node(
                physical_index_maybe,
                blocks,
                set_blocks,
                next_id,
                set_next_id,
                InnerBlockType::Lacuna,
            );
        };
    });

    view! {
        <div>
        <button on:click=add_block>"Add a new thingy"</button>
        <br/>
        <For each=move || blocks.get()
            key=|block| block.id
            let(EditorBlock {id, inner, focus_on_load, })
            >
            <br/>
            <div>
            {move || move_down_button(id)}
            <span>{id}":"{move || inner.clone().view(id, focus_on_load)}</span>
            <button on:click=move |_| set_blocks.write().retain(|blck| blck.id != id)>"Remove this thingy!"</button>
            {move || move_up_button(id)}
            </div>
        </For>
        </div>
    }
}


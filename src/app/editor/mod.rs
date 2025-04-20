//! Everything needed for the raw editor functionality is in this module.
//!
//! This is the GUI-area and directly related APIs/server functions to save its data.

use leptos::{ev::keydown, logging::log, prelude::*};
use leptos_use::{use_document, use_event_listener};
use undo::{BlockDeletion, BlockInsertion, BlockSwap, UnReStack, UnReStep};
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement};

mod blocks;
use blocks::*;

mod undo;

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
    // get the block index we are in right now
    // break if this is an ID which we do not know
    if !primary_input.id().starts_with("block-input-") {
        return;
    };
    let id_stripped = &primary_input.id()[12..];
    let id = match id_stripped.parse::<i32>() {
        Ok(el) => el,
        Err(_) => {
            return;
        }
    };

    let current_select_start = primary_input.selection_start().unwrap_or(None);
    let current_select_end = primary_input.selection_end().unwrap_or(None);
    let complete_value = primary_input.value();
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
                    // we want to focus on the next node
                    EditorBlock::new(next_id.get(), block_type, String::default(), true),
                );
                *set_next_id.write() += 1;
            }
        };
    };
}

#[component]
pub(crate) fn Editor() -> impl IntoView {
    let undo_stack = RwSignal::new(UnReStack::new());

    let initial_id = 1;
    let (next_id, set_next_id) = signal(initial_id);
    let init_blocks = Vec::<EditorBlock>::new(); 
    let (blocks, set_blocks) = signal(init_blocks);
    let add_block = move |_| {
        set_blocks.update(|bs| {
            let logical_index = next_id.get();
            let new_block = EditorBlockDry::new(
                logical_index,
                InnerBlockType::Text,
                "raw text".to_owned(),
                true,
                );
            let physical_index = bs.len();
            undo_stack.write().push_undo(UnReStep::BlockInsertion(BlockInsertion::new(physical_index, new_block.clone())));
            bs.push(new_block.into());
            set_next_id.update(|idx| *idx += 1);
        })
    };

    let physical_index_maybe = move |id: i32| blocks.read().iter().position(|b| b.id() == id);

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
                    // push the swap to the undo stack
                    undo_stack.write().push_undo(UnReStep::BlockSwap(BlockSwap::new(physical_index, physical_index - 1)));
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
                    // swap them
                    set_blocks.write().swap(physical_index, physical_index + 1);
                    // push the swap to the undo stack
                    undo_stack.write().push_undo(UnReStep::BlockSwap(BlockSwap::new(physical_index, physical_index + 1)));
                }>"Move this thingy down"</button>
            })
        } else {
            None
        }
    };

    // the keyboard-shortcut listener
    let _cleanup = use_event_listener(use_document(), keydown, move |evt| {
        log!("{}", evt.key_code());
        // <ctrl>-<alt>-Z - undo
        if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 90 {
            match undo_stack.write().undo(&mut set_blocks.write()) {
                Ok(()) => {}
                Err(e) => {
                    log!("{e}");
                }
            };
        // <ctrl>-<alt>-R - redo
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 82 {
            match undo_stack.write().redo(&mut set_blocks.write()) {
                Ok(()) => {}
                Err(e) => {
                    log!("{e}");
                }
            };
        // <ctrl>-<alt>-T (new Text)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 84 {
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
        // <ctrl>-<alt>-<ENTER> (new Break)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 13 {
            new_node(
                physical_index_maybe,
                blocks,
                set_blocks,
                next_id,
                set_next_id,
                InnerBlockType::Break,
            );
        };
    });

    // the undo_stack is used in most inner blocks later and we do not want to manually pass it
    // around
    provide_context(undo_stack);

    view! {
        <div>
        <button on:click=add_block>"Add a new thingy"</button>
        <br/>
        <For each=move || blocks.get()
            key=|block| block.id()
            children={move |outer_block|
                {
                let outer_id = outer_block.id();
                view!{
                    <br/>
                    <div>
                    {move || move_down_button(outer_id)}
                    {move || { outer_block.clone().view() }}
                    <button on:click=move |_| {
                        let physical_index = match blocks.read().iter().position(|blck| blck.id() == outer_id) {
                            Some(x) => x,
                            // the given element does not exist - this should be impossible
                            None => { return; }
                        };
                        // remove this element
                        let removed_block = set_blocks.write().remove(physical_index);
                        // push this action to the undo stack
                        undo_stack.write().push_undo(UnReStep::BlockDeletion(BlockDeletion::new(physical_index, removed_block.into())));
                    }>"Remove this thingy!"</button>
                    {move || move_up_button(outer_id)}
                    </div>
                }
                }
            }
        >
        </For>
        </div>
    }
}

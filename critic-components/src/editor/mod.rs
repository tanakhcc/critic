//! Everything needed for the raw editor functionality is in this module.
//!
//! This is the GUI-area and directly related APIs/server functions to save its data.

use critic_format::streamed::BlockType;
use leptos::{ev::keydown, logging::log, prelude::*};
use leptos_use::{use_document, use_event_listener};
use undo::{UnReStack, UnReStep};
use web_sys::{wasm_bindgen::JsCast, HtmlTextAreaElement};

pub mod blocks;
use blocks::*;

mod undo;

mod versification_scheme;

/// Add a new Block to the editor
///
/// `blocks`: the blocks currently present
/// `next_id`: use this ID for the new block
/// `block_type`: create a block of this type
/// `undo_stack`: add an undo-action for the block creation to this [`UnReStack`]
/// `default_language`: use this language for the new block if its language cannot be determined
/// automatically
fn new_node(
    blocks: RwSignal<Vec<EditorBlock>>,
    next_id: RwSignal<usize>,
    block_type: BlockType,
    undo_stack: RwSignal<UnReStack>,
    default_language: &str,
) {
    // first find out the id of the block currently selected
    let active_element = match use_document().active_element() {
        Some(el) => el,
        None => {
            return;
        }
    };
    let primary_input = match active_element.dyn_into::<HtmlTextAreaElement>() {
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
    let id = match id_stripped.parse::<usize>() {
        Ok(el) => el,
        Err(_) => {
            return;
        }
    };

    let physical_index_maybe = move |id: usize| blocks.read().iter().position(|b| b.id() == id);

    // If text is currently selected, the block should be created with the selected text as its
    // content
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
                    Some(el) => {
                        let res = el.split_at_selection(
                            start_utf8,
                            end_utf8,
                            block_type,
                            &mut next_id.write(),
                        );
                        res
                    }
                    None => {
                        return;
                    }
                };
                // replace the block currently at physical_index with the new blocks
                let removed = blocks
                    .write()
                    .splice(physical_index..physical_index + 1, new_blocks.clone())
                    .collect();
                // add the change to the undo stack
                undo_stack.write().push_undo(UnReStep::new_block_change(
                    physical_index,
                    removed,
                    new_blocks.into_iter().collect(),
                ));
            }
            _ => {
                // nothing selected, add a new empty node after this one
                // we want to focus on the next node
                let new_block = EditorBlock::new(
                    next_id.get(),
                    block_type,
                    default_language.to_string(),
                    String::default(),
                    true,
                );
                blocks.write().insert(physical_index, new_block.clone());
                // add the insertion to the undo stack
                undo_stack
                    .write()
                    .push_undo(UnReStep::new_insertion(physical_index, new_block));
                *next_id.write() += 1;
            }
        };
    };
}

/// The raw block-editor (i.e. not containing XML and such)
#[component]
pub fn Editor(
    blocks: RwSignal<Vec<EditorBlock>>,
    default_language: String,
    on_save: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
) -> impl IntoView {
    let undo_stack = RwSignal::new(UnReStack::new());

    // logical ID (insertion order) of blocks, 1-based
    let next_id = RwSignal::new(blocks.read_untracked().len() + 1);

    let physical_index_maybe = move |id: usize| blocks.read().iter().position(|b| b.id() == id);

    let index_if_not_first =
        move |id: usize| physical_index_maybe(id).filter(|&physical_index| physical_index != 0);

    let move_up_button = move |id| {
        if let Some(physical_index) = index_if_not_first(id) {
            view! {
                <button on:click=move |_| {
                    blocks.write().swap(physical_index, physical_index - 1);
                    undo_stack
                        .write()
                        .push_undo(UnReStep::new_swap(physical_index, physical_index - 1));
                }>
                    // move up
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M4.5 10.5 12 3m0 0 7.5 7.5M12 3v18"
                        />
                    </svg>
                </button>
            }
            .into_any()
        } else {
            view! {
                <button on:click=move |_| {} disabled=true>
                    // move up
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="text-gray-300 size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M4.5 10.5 12 3m0 0 7.5 7.5M12 3v18"
                        />
                    </svg>
                </button>
            }
            .into_any()
        }
    };

    let index_if_not_last = move |id: usize| {
        physical_index_maybe(id).filter(|&physical_index| physical_index < blocks.read().len() - 1)
    };

    let move_down_button = move |id| {
        if let Some(physical_index) = index_if_not_last(id) {
            view! {
                <button on:click=move |_| {
                    blocks.write().swap(physical_index, physical_index + 1);
                    undo_stack
                        .write()
                        .push_undo(UnReStep::new_swap(physical_index, physical_index + 1));
                }>
                    // move down
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M19.5 13.5 12 21m0 0-7.5-7.5M12 21V3"
                        />
                    </svg>
                </button>
            }
            .into_any()
        } else {
            view! {
                <button on:click=move |_| {} disabled=true>
                    // move down
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="text-gray-300 size-6"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M19.5 13.5 12 21m0 0-7.5-7.5M12 21V3"
                        />
                    </svg>
                </button>
            }
            .into_any()
        }
    };

    // the keyboard-shortcut listener
    let cloned_default_language = default_language.clone();
    let _cleanup = use_event_listener(use_document(), keydown, move |evt| {
        // <ctrl>-<alt>-S - Save
        if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 83 {
            // we can only dispatch and hope for the best here
            on_save.dispatch(blocks.read().to_owned());
        // <ctrl>-<alt>-Z - undo
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 90 {
            match undo_stack.write().undo(&mut blocks.write()) {
                Ok(()) => {}
                Err(e) => {
                    log!("{e}");
                }
            };
        // <ctrl>-<alt>-R - redo
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 82 {
            match undo_stack.write().redo(&mut blocks.write()) {
                Ok(()) => {}
                Err(e) => {
                    log!("{e}");
                }
            };
        // <ctrl>-<alt>-T (new Text)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 84 {
            new_node(
                blocks,
                next_id,
                BlockType::Text,
                undo_stack,
                &cloned_default_language,
            );
        // <ctrl>-<alt>-A (new Abbreviation)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 65 {
            new_node(
                blocks,
                next_id,
                BlockType::Abbreviation,
                undo_stack,
                &cloned_default_language,
            )
        // <ctrl>-<alt>-U (new Uncertain)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 85 {
            new_node(
                blocks,
                next_id,
                BlockType::Uncertain,
                undo_stack,
                &cloned_default_language,
            )
        // <ctrl>-<alt>-L (new Lacuna)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 76 {
            new_node(
                blocks,
                next_id,
                BlockType::Lacuna,
                undo_stack,
                &cloned_default_language,
            );
        // <ctrl>-<alt>-V (new Anchor/Verse)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 86 {
            new_node(
                blocks,
                next_id,
                BlockType::Anchor,
                undo_stack,
                &cloned_default_language,
            );
        // <ctrl>-<alt>-C (new Correction)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 67 {
            new_node(
                blocks,
                next_id,
                BlockType::Correction,
                undo_stack,
                &cloned_default_language,
            );
        // <ctrl>-<alt>-<space> (new Space)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 32 {
            new_node(
                blocks,
                next_id,
                BlockType::Space,
                undo_stack,
                &cloned_default_language,
            );
        // <ctrl>-<alt>-<ENTER> (new Break)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 13 {
            new_node(
                blocks,
                next_id,
                BlockType::Break,
                undo_stack,
                &cloned_default_language,
            );
        };
    });

    // the undo_stack is used in most inner blocks later and we do not want to manually pass it
    // around
    provide_context(undo_stack);

    // Start loading versification schemes and provide them - only the Anchor components will use
    // them, and probably only much later then page load
    let versification_schemes =
        OnceResource::new(versification_scheme::get_versification_schemes());
    provide_context(versification_schemes);

    view! {
        <EditorEditButtons
            default_language=default_language
            blocks=blocks
            next_id=next_id
            undo_stack=undo_stack
            on_save=on_save
        />
        <div id="editor-blocks" class="h-0 grow overflow-y-auto">
            <For
                each=move || blocks.get()
                key=|block| block.id()
                children=move |outer_block| {
                    let outer_id = outer_block.id();
                    view! {
                        <br />
                        <div class="flex justify-between">
                            <span>
                                {move || move_up_button(outer_id)}
                                {move || move_down_button(outer_id)}
                            </span>

                            {move || { outer_block.clone().view() }}

                            <button on:click=move |_| {
                                let physical_index = match blocks
                                    .read()
                                    .iter()
                                    .position(|blck| blck.id() == outer_id)
                                {
                                    Some(x) => x,
                                    None => {
                                        return;
                                    }
                                };
                                let removed_block = blocks.write().remove(physical_index);
                                undo_stack
                                    .write()
                                    .push_undo(
                                        UnReStep::new_deletion(physical_index, removed_block),
                                    );
                            }>
                                <svg
                                    xmlns="http://www.w3.org/2000/svg"
                                    fill="none"
                                    viewBox="0 0 24 24"
                                    stroke-width="1.5"
                                    stroke="currentColor"
                                    class="size-6"
                                >
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        d="M12 9.75 14.25 12m0 0 2.25 2.25M14.25 12l2.25-2.25M14.25 12 12 14.25m-2.58 4.92-6.374-6.375a1.125 1.125 0 0 1 0-1.59L9.42 4.83c.21-.211.497-.33.795-.33H19.5a2.25 2.25 0 0 1 2.25 2.25v10.5a2.25 2.25 0 0 1-2.25 2.25h-9.284c-.298 0-.585-.119-.795-.33Z"
                                    />
                                </svg>
                            </button>
                        </div>
                    }
                }
            ></For>
        </div>
    }
}

#[component]
fn EditorEditButtons(
    blocks: RwSignal<Vec<EditorBlock>>,
    next_id: RwSignal<usize>,
    undo_stack: RwSignal<UnReStack>,
    default_language: String,
    on_save: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
) -> impl IntoView {
    const BUTTON_DEFAULT_CLASS: &str = "rounded-md bg-slate-700 p-1 hover:bg-slate-500";

    // each on click handler needs to own the default language, so we arc-clone it :/
    let default_language = std::sync::Arc::new(default_language);
    let text_lang = default_language.clone();
    let uncertain_lang = default_language.clone();
    let lacuna_lang = default_language.clone();
    let abbr_lang = default_language.clone();
    let corr_lang = default_language.clone();
    let space_lang = default_language.clone();
    let break_lang = default_language.clone();
    view! {
        <div class="grid grid-cols-11 gap-1 border-b border-slate-600 p-1" id="editor-tab-header">
            <span class="text-orange-400 flex flex-col justify-center">ctrl + alt +</span>
            <button class=BUTTON_DEFAULT_CLASS>
                <span
                    class="text-orange-400"
                    on:click=move |ev| {
                        ev.prevent_default();
                        on_save.dispatch(blocks.read().to_owned());
                    }
                >
                    "S: "
                </span>
                save
            </button>
            <button class=BUTTON_DEFAULT_CLASS>
                <span
                    class="text-orange-400"
                    on:click=move |_ev| {
                        match undo_stack.write().undo(&mut blocks.write()) {
                            Ok(()) => {}
                            Err(e) => {
                                log!("{e}");
                            }
                        };
                    }
                >
                    "Z: "
                </span>
                undo
            </button>
            <button class=BUTTON_DEFAULT_CLASS>
                <span
                    class="text-orange-400"
                    on:click=move |_ev| {
                        match undo_stack.write().redo(&mut blocks.write()) {
                            Ok(()) => {}
                            Err(e) => {
                                log!("{e}");
                            }
                        };
                    }
                >
                    "R: "
                </span>
                redo
            </button>
            <button
                class=BUTTON_DEFAULT_CLASS
                on:mousedown=move |ev| {
                    ev.prevent_default();
                    new_node(blocks, next_id, BlockType::Text, undo_stack, &text_lang);
                }
            >
                <span class="text-orange-400">"T: "</span>
                text
            </button>
            <button
                class=BUTTON_DEFAULT_CLASS
                on:mousedown=move |ev| {
                    ev.prevent_default();
                    new_node(blocks, next_id, BlockType::Uncertain, undo_stack, &uncertain_lang);
                }
            >
                <span class="text-orange-400">"U: "</span>
                uncertain
            </button>
            <button
                class=BUTTON_DEFAULT_CLASS
                on:mousedown=move |ev| {
                    ev.prevent_default();
                    new_node(blocks, next_id, BlockType::Lacuna, undo_stack, &lacuna_lang);
                }
            >
                <span class="text-orange-400">"L: "</span>
                lacuna
            </button>
            <button
                class=BUTTON_DEFAULT_CLASS
                on:mousedown=move |ev| {
                    ev.prevent_default();
                    new_node(blocks, next_id, BlockType::Abbreviation, undo_stack, &abbr_lang);
                }
            >
                <span class="text-orange-400">"A: "</span>
                abbreviation
            </button>
            <button
                class=BUTTON_DEFAULT_CLASS
                on:mousedown=move |ev| {
                    ev.prevent_default();
                    new_node(blocks, next_id, BlockType::Correction, undo_stack, &corr_lang);
                }
            >
                <span class="text-orange-400">"C: "</span>
                correction
            </button>
            <button
                class="inline-flex rounded-md bg-slate-700 p-1 hover:bg-slate-500"
                on:mousedown=move |ev| {
                    ev.prevent_default();
                    new_node(blocks, next_id, BlockType::Space, undo_stack, &space_lang);
                }
            >
                <span class="inline-flex text-orange-400">
                    <svg
                        class="size-4 translate-y-2"
                        viewBox="0 0 24 24"
                        version="1.1"
                        xmlns="http://www.w3.org/2000/svg"
                        xmlns:xlink="http://www.w3.org/1999/xlink"
                    >
                        <g stroke="none" stroke-width="1" fill="none" fill-rule="evenodd">
                            <g fill="currentColor" fill-rule="nonzero">
                                <path d="M20.5,11 L20.5,13 C20.5,13.1380712 20.3880712,13.25 20.25,13.25 L3.75,13.25 C3.61192881,13.25 3.5,13.1380712 3.5,13 L3.5,11 C3.5,10.5857864 3.16421356,10.25 2.75,10.25 C2.33578644,10.25 2,10.5857864 2,11 C2,11.4444444 2,12.1111111 2,13 C2,13.9664983 2.78350169,14.75 3.75,14.75 L20.25,14.75 C21.2164983,14.75 22,13.9664983 22,13 L22,11 C22,10.5857864 21.6642136,10.25 21.25,10.25 C20.8357864,10.25 20.5,10.5857864 20.5,11 Z"></path>
                            </g>
                        </g>
                    </svg>
                    :
                </span>
                "space"
            </button>
            <button
                class="inline-flex rounded-md bg-slate-700 p-1 hover:bg-slate-500"
                on:click=move |ev| {
                    ev.prevent_default();
                    new_node(blocks, next_id, BlockType::Break, undo_stack, &break_lang);
                }
            >
                <span class="inline-flex text-orange-400">
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="size-4 translate-y-1.5"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="m7.49 12-3.75 3.75m0 0 3.75 3.75m-3.75-3.75h16.5V4.499"
                        />
                    </svg>
                    :
                </span>
                enter
            </button>
        </div>
    }
}

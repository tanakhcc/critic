//! Everything needed for the raw editor functionality is in this module.
//!
//! This is the GUI-area and directly related APIs/server functions to save its data.

use critic_format::streamed::BlockType;
use leptos::{
    ev::keydown,
    logging::log,
    prelude::{Action, *},
};
use leptos_use::{use_document, use_event_listener};
use save::{load_editor_state, save_editor_state};
use undo::{UnReStack, UnReStep};
use web_sys::{wasm_bindgen::JsCast, HtmlTextAreaElement};

mod blocks;
use blocks::*;

mod undo;

mod save;

/// Add a new Block to the editor
///
/// `physical_index_maybe`: find the physical position of the block with this id
/// `blocks`: the blocks currently present
/// `next_id`: use this ID for the new block
/// `block_type`: create a block of this type
/// `undo_stack`: add an undo-action for the block creation to this [`UnReStack`]
/// `default_language`: use this language for the new block if its language cannot be determined
/// automatically
fn new_node(
    physical_index_maybe: impl Fn(usize) -> Option<usize>,
    blocks: ReadSignal<Vec<EditorBlock>>,
    set_blocks: WriteSignal<Vec<EditorBlock>>,
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

    // If text is currently selected, the block should be created with the selected text as its
    // content
    let current_select_start = primary_input.selection_start().unwrap_or(None);
    let current_select_end = primary_input.selection_end().unwrap_or(None);
    let complete_value = primary_input.value();
    // split this element in blocks
    if let Some(physical_index) = physical_index_maybe(id) {
        leptos::logging::log!("physical_index of selected block: {physical_index}");
        // this one block will usually be split in three: before-selection, selection,
        // after-selection
        match (current_select_start, current_select_end) {
            (Some(x), Some(y)) => {
                leptos::logging::log!("start and end of text selection: {x}, {y}");
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
                        leptos::logging::log!(
                            "the block that was selected from the physical index: {el:?}"
                        );
                        leptos::logging::log!("all blocks: {:?}", blocks.get_untracked());
                        leptos::logging::log!("next ID to give out: {:?}", next_id.get());
                        let res = el.split_at_selection(
                            start_utf8,
                            end_utf8,
                            block_type,
                            &mut next_id.write(),
                        );
                        leptos::logging::log!("result from split_at_selection: {res:?}");
                        res
                    }
                    None => {
                        return;
                    }
                };
                // replace the block currently at physical_index with the new blocks
                let removed = set_blocks
                    .write()
                    .splice(physical_index..physical_index + 1, new_blocks.clone())
                    .map(|b| b.into())
                    .collect();
                // add the change to the undo stack
                undo_stack.write().push_undo(UnReStep::new_block_change(
                    physical_index,
                    removed,
                    new_blocks.into_iter().map(|x| x.into()).collect(),
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
                set_blocks
                    .write()
                    .insert(physical_index, new_block.clone().into());
                // add the insertion to the undo stack
                undo_stack
                    .write()
                    .push_undo(UnReStep::new_insertion(physical_index, new_block));
                *next_id.write() += 1;
            }
        };
    };
}

#[component]
pub(crate) fn Editor(default_language: String) -> impl IntoView {
    let undo_stack = RwSignal::new(UnReStack::new());

    // logical ID of blocks, 1-based
    let next_id = RwSignal::new(1_usize);
    let init_blocks = Vec::<EditorBlock>::new();
    let (blocks, set_blocks) = signal(init_blocks);
    let add_blocks_lang = default_language.clone();
    let add_block = move |_| {
        set_blocks.update(|bs| {
            let logical_index = next_id.get();
            let new_block = EditorBlock::new(
                logical_index,
                BlockType::Text,
                add_blocks_lang.clone(),
                "raw text".to_owned(),
                true,
            );
            let physical_index = bs.len();
            undo_stack
                .write()
                .push_undo(UnReStep::new_insertion(physical_index, new_block.clone()));
            bs.push(new_block.into());
            next_id.update(|idx| *idx += 1);
        })
    };

    let physical_index_maybe = move |id: usize| blocks.read().iter().position(|b| b.id() == id);

    let index_if_not_first = move |id: usize| {
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
            view! {
                <button
                    on:click=move |_| {
                    set_blocks.write().swap(physical_index, physical_index - 1);
                    // push the swap to the undo stack
                    undo_stack.write().push_undo(UnReStep::new_swap(physical_index, physical_index - 1));
                }>
                // move up
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 10.5 12 3m0 0 7.5 7.5M12 3v18" />
</svg>
                </button>
            }.into_any()
        } else {
            view! {
                <button
                    on:click=move |_| {} disabled=true>
                // move up
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="text-gray-300 size-6">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 10.5 12 3m0 0 7.5 7.5M12 3v18" />
</svg>
                </button>
            }.into_any()
        }
    };

    let index_if_not_last = move |id: usize| {
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
            view! {
                <button
                    on:click=move |_| {
                    // swap them
                    set_blocks.write().swap(physical_index, physical_index + 1);
                    // push the swap to the undo stack
                    undo_stack.write().push_undo(UnReStep::new_swap(physical_index, physical_index + 1));
                }>
                // move down
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 13.5 12 21m0 0-7.5-7.5M12 21V3" />
</svg>
                </button>
            }.into_any()
        } else {
            view! {
                <button on:click=move |_| {} disabled=true>
                // move down
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="text-gray-300 size-6">
  <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 13.5 12 21m0 0-7.5-7.5M12 21V3" />
</svg>
                </button>
            }.into_any()
        }
    };

    let save_state_action = Action::new(|blocks: &Vec<EditorBlock>| {
        let blocks_dehydrated = blocks.iter().map(|b| b.inner.clone().into()).collect();
        async move { save_editor_state(blocks_dehydrated).await }
    });
    let pending_save = save_state_action.pending();

    // the keyboard-shortcut listener
    let _cleanup = use_event_listener(use_document(), keydown, move |evt| {
        log!("{}", evt.key_code());
        // <ctrl>-<alt>-S - Save
        if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 83 {
            // we can only dispatch and hope for the best here
            save_state_action.dispatch(blocks.read().to_owned());
        // <ctrl>-<alt>-Z - undo
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 90 {
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
                BlockType::Text,
                undo_stack,
                &default_language,
            );
        // <ctrl>-<alt>-U (new Uncertain)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 85 {
            new_node(
                physical_index_maybe,
                blocks,
                set_blocks,
                next_id,
                BlockType::Uncertain,
                undo_stack,
                &default_language,
            )
        // <ctrl>-<alt>-L (new Lacuna)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 76 {
            new_node(
                physical_index_maybe,
                blocks,
                set_blocks,
                next_id,
                BlockType::Lacuna,
                undo_stack,
                &default_language,
            );
        // <ctrl>-<alt>-<ENTER> (new Break)
        } else if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 13 {
            new_node(
                physical_index_maybe,
                blocks,
                set_blocks,
                next_id,
                BlockType::Break,
                undo_stack,
                &default_language,
            );
        };
    });

    // the undo_stack is used in most inner blocks later and we do not want to manually pass it
    // around
    provide_context(undo_stack);

    let load_state_resource = OnceResource::<Vec<EditorBlock>>::new(async move {
        match load_editor_state().await {
            Ok(streamed) => {
                let blocks: Vec<EditorBlock> = streamed
                    .content
                    .into_iter()
                    .enumerate()
                    .map(|(idx, x)| EditorBlock {
                        focus_on_load: false,
                        inner: x.into(),
                        id: idx,
                    })
                    .collect();
                *next_id.write() = blocks.len() + 1;
                blocks
            }
            Err(e) => {
                log!("Error loading server state: {e}");
                vec![]
            }
        }
    });

    view! {
            <div>
            <button on:click=add_block>"Add a new thingy"</button>
            <button on:click=move |_| {
                save_state_action.dispatch(blocks.read().to_owned());
            }>"Save state"</button>
            <p>{move || pending_save.get().then_some("Saving state...")}</p>
            <br/>
            <Suspense fallback=|| { view!{ <p>"Loading editor state from the server..."</p> } }>
            {move || Suspend::new(async move {
                let init_blocks = load_state_resource.await;
                *next_id.write() = init_blocks.len() + 1;
                set_blocks.set(init_blocks.into_iter().map(|b| b.into()).collect());
            view!{
            <For each=move || blocks.get()
                key=|block| block.id()
                children={move |outer_block|
                    {
                    let outer_id = outer_block.id();
                    view!{
                        <br/>
                        <div class="flex justify-between">
                        <span>
                        {move || move_up_button(outer_id)}
                        {move || move_down_button(outer_id)}
                        </span>

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
                            undo_stack.write().push_undo(UnReStep::new_deletion(physical_index, removed_block.into()));
                        }>
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9.75 14.25 12m0 0 2.25 2.25M14.25 12l2.25-2.25M14.25 12 12 14.25m-2.58 4.92-6.374-6.375a1.125 1.125 0 0 1 0-1.59L9.42 4.83c.21-.211.497-.33.795-.33H19.5a2.25 2.25 0 0 1 2.25 2.25v10.5a2.25 2.25 0 0 1-2.25 2.25h-9.284c-.298 0-.585-.119-.795-.33Z" />
    </svg>
                        </button>
                        </div>
                    }
                    }
                }
            >
            </For>
            }})}
            </Suspense>
            </div>
        }
}

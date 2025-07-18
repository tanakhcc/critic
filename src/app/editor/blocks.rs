//! The Types and associated functions for Blocks
//!
//! This module defines, what blocks are available, do and look like. Interaction with other
//! elements is handled in [`editor`](crate::app::editor) itself.

use critic_format::streamed::{
    Abbreviation, Anchor, Block, BlockType, BreakType, Correction, FromTypeLangAndContent, Lacuna,
    Paragraph, Space, Uncertain, Version,
};
use leptos::{html::Textarea, prelude::*};
use serde::{Deserialize, Serialize};

use super::{versification_scheme::VersificationScheme, UnReStack, UnReStep};

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
    view! {
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
                    id={format!("block-input-{id}-language")}
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
            id={format!("block-input-{id}-reason")}
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
                    id={format!("block-input-{id}-extent")}
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
                    id={format!("block-input-{id}-unit")}
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
                    <option value="Character">Character</option>
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
            id={format!("block-input-{id}-agent")}
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
                    id={format!("block-input-{id}-language")}
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
                    id={format!("block-input-{id}-certainty")}
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

fn inner_space_view(
    undo_stack: RwSignal<UnReStack>,
    space: RwSignal<Space>,
    id: usize,
) -> impl IntoView {
    let current_space = RwSignal::new(space.get_untracked());
    view! {
        <div class="flex justify-between">
        <span
            class="font-light text-xs">
                "Space: "
        </span>
                    <span class="font-light text-xs">"Extent: "</span>
                    <input
                    prop:value=move || space.read().quantity.clone()
                    class="text-sm"
                    placeholder="extent"
                    autocomplete="false"
                    spellcheck="false"
                    id={format!("block-input-{id}-extent")}
                    on:input:target=move |ev| {
                        // here we can allow the value to be unparsable, but we want to prevent
                        // this if possible
                        let x = ev.target().value();
                        if x.is_empty() {
                        } else {
                            space.write().quantity = ev.target().value().parse().unwrap_or_else(|_| 1);
                        }
                    }
                    on:change:target=move |ev| {
                        // just throw away values that are not parsable
                        space.write().quantity = ev.target().value().parse().unwrap_or_else(|_| 1);
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Space(current_space.get_untracked()),
                                Block::Space(space.get_untracked().into()))
                            );
                        // now set the new savepoint
                        current_space.write().quantity = space.get_untracked().quantity;
                    }/>
                    <span class="font-light text-xs">"Unit of Extent: "</span>
                    <select
                    id={format!("block-input-{id}-unit")}
                    prop:value=move || space.read().unit.name()
                    on:input:target=move |ev| {
                        space.write().unit = ev.target().value().parse().expect("Only correct Names in the options for this select field.");
                    }
                    on:change:target=move |ev| {
                        space.write().unit = ev.target().value().parse().expect("Only correct Names in the options for this select field.");
                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Space(current_space.get_untracked()),
                                Block::Space(space.get_untracked())));
                        current_space.write().unit = space.get_untracked().unit;
                    }
                >
                    <option value="Character">Character</option>
                    <option value="Line">Line</option>
                    <option value="Column">Column</option>
                </select>
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

/// display an anchor
fn inner_anchor_view(
    undo_stack: RwSignal<UnReStack>,
    anchor: RwSignal<Anchor>,
    id: usize,
) -> impl IntoView {
    let current_anchor = RwSignal::new(anchor.get_untracked());

    let config_expanded = signal(false);
    let Some(versification_schemes_res) =
        use_context::<OnceResource<Result<Vec<VersificationScheme>, ServerFnError>>>()
    else {
        leptos::logging::log!(
            "Did not get a provided context for versification schemes. Please open a bug report."
        );
        return leptos::either::Either::Left(view! {
            <p>"No versification schemes present. Anchor cannot be represented! Please open a bug report."</p>
        });
    };

    let raw_id = RwSignal::new(if !anchor.read_untracked().anchor_id.starts_with("A_V_") {
        String::default()
    } else {
        anchor.read_untracked().anchor_id[4..]
            .split("_")
            .nth(2)
            .map_or(String::default(), |substr| substr.to_string())
    });

    // The shorthand associated to the currently selected versification scheme, which is needed
    // both when changing the id as well as the type
    let scheme_shorthand = Memo::new(move |_| 'await_response: loop {
        match versification_schemes_res.get() {
            Some(Ok(schemes)) => {
                for scheme in schemes {
                    if scheme.full_name == anchor.read().anchor_type {
                        break 'await_response scheme.shorthand;
                    }
                }
                leptos::logging::log!("Did get versification schemes, but could not find the short hand form for long form: {}", anchor.read().anchor_type);
                break 'await_response "???".to_string();
            }
            _ => {
                // wait until the server responds with the versification schemes
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    });

    leptos::either::Either::Right(view! {
        <div class="flex justify-between">
        <div>
            // Anchor 'content', i.e. the actual id not containing the versification scheme
            <input
            prop:value=move || raw_id.get()
            class="text-sm"
            placeholder="id"
            autocomplete="false"
            spellcheck="false"
            id={format!("block-input-{id}-anchor_id")}
            on:input:target=move |ev| {
                // just set the raw input value
                *raw_id.write() = ev.target().value();
            }
            on:change:target=move |ev| {
                *raw_id.write() = ev.target().value();

                let full_anchor_id = format!("A_V_{}_{}", scheme_shorthand.get(), raw_id.read());
                anchor.write().anchor_id = full_anchor_id;
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Anchor(current_anchor.get_untracked()),
                        Block::Anchor(anchor.get_untracked().into()))
                    );
                // now set the new savepoint
                current_anchor.write().anchor_id = anchor.get_untracked().anchor_id;
            }/>
        </div>
        <Accordion
            expand={config_expanded}
            expanded={Box::new(|| view! { <CogSvg/> }.into_any())}
            collapsed={Box::new(|| view! { <CogSvg/> }.into_any())}
        >
            <List>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Versification Scheme: "</span>
                    <select
                    prop:value=move || anchor.read().anchor_type.clone()
                    on:input:target=move |ev| {
                        anchor.write().anchor_type = ev.target().value();
                    }
                    on:change:target=move |ev| {
                        anchor.write().anchor_type = ev.target().value();

                        // we also need to update the anchor id with the new shorthand for the
                        // scheme when the anchor type is changed
                        let full_anchor_id = format!("A_V_{}_{}", scheme_shorthand.get(), raw_id.read());
                        anchor.write().anchor_id = full_anchor_id;

                        undo_stack.write().push_undo(UnReStep::new_data_change(id,
                                Block::Anchor(current_anchor.get_untracked()),
                                Block::Anchor(anchor.get_untracked())));
                        current_anchor.write().anchor_type = anchor.get_untracked().anchor_type;
                    }
                >
                    // these two schemes are static and can always be show.
                    // please change if you change the static schemes in
                    // `202507071848_versification_scheme.up.sql`
                    <Suspense fallback = move || view!{ <option value="Present">Present</option><option value="Common">Common</option>}>
                    {
                        versification_schemes_res.get_untracked().map(|scheme_res|
                            match scheme_res {
                                Ok(schemes) => {
                                    leptos::either::Either::Left(
                                        schemes.into_iter().map(|scheme|
                                            view! {
                                                <option value=scheme.full_name>{scheme.full_name.clone()}</option>
                                            }).collect::<Vec<_>>())
                                }
                                Err(e) => {
                                    leptos::either::Either::Right(
                                    view!{
                                        <p>"Unable to get versification schemes from the server: "{e.to_string()}". Please open a bug report."</p>
                                    })
                                }
                            })
                    }
                    </Suspense>
                </select>
                </Item>
            </List>
        </Accordion>
        </div>
    })
}

/// View for an abbreviation
fn inner_abbreviation_view(
    undo_stack: RwSignal<UnReStack>,
    abbreviation: RwSignal<Abbreviation>,
    focus_element: leptos::prelude::NodeRef<Textarea>,
    id: usize,
) -> impl IntoView {
    let current_abbreviation = RwSignal::new(abbreviation.get_untracked());

    let expansion_config_expanded = signal(false);
    let surface_config_expanded = signal(false);
    view! {
        <div class="flex justify-between">
        <span>
            "Surface form:"
        </span>
        <div>
            // surface form
            <textarea
            class="bg-orange-100 text-black font-mono"
            node_ref=focus_element
            prop:value=move || abbreviation.read().surface.clone()
            autocomplete="false"
            spellcheck="false"
            id={format!("block-input-{id}-surface")}
            rows=1
            cols=TEXTAREA_DEFAULT_COLS
            on:input:target=move |ev| {
                abbreviation.write().surface = ev.target().value();
            }
            on:change:target=move |ev| {
                abbreviation.write().surface = ev.target().value();
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Abbreviation(current_abbreviation.get_untracked()),
                        Block::Abbreviation(abbreviation.get_untracked().into()))
                    );
                // now set the new savepoint
                current_abbreviation.write().surface = abbreviation.get_untracked().surface;
            }
        />
        </div>
        <Accordion
            expand={surface_config_expanded}
            expanded={Box::new(|| view! { <CogSvg/> }.into_any())}
            collapsed={Box::new(|| view! { <CogSvg/> }.into_any())}
        >
            <List>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Surface Language: "</span>
                    <input
                    prop:value=move || abbreviation.read().surface_lang.clone()
                    class="text-sm"
                    placeholder="surface-language"
                    autocomplete="false"
                    spellcheck="false"
                    id={format!("block-input-{id}-surface_lang")}
                    on:input:target=move |ev| {
                        abbreviation.write().surface_lang = ev.target().value();
                    }
                    on:change:target=move |ev| {
                        abbreviation.write().surface_lang= ev.target().value();
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Abbreviation(current_abbreviation.get_untracked()),
                                Block::Abbreviation(abbreviation.get_untracked().into()))
                            );
                        // now set the new savepoint
                        current_abbreviation.write().surface_lang = abbreviation.read_untracked().surface_lang.clone();
                    }/>
                </Item>
            </List>
        </Accordion>
        </div>

        <div class="flex justify-between">
        <span>
            "Expanded form:"
        </span>
        <div>
            // expanded form
            <textarea
            class="bg-orange-100 text-black font-mono"
            id={format!("block-input-{id}")}
            node_ref=focus_element
            prop:value=move || abbreviation.read().expansion.clone()
            autocomplete="false"
            spellcheck="false"
            rows=1
            cols=TEXTAREA_DEFAULT_COLS
            on:input:target=move |ev| {
                abbreviation.write().expansion = ev.target().value();
            }
            on:change:target=move |ev| {
                abbreviation.write().expansion = ev.target().value();
                undo_stack.write().push_undo(
                    UnReStep::new_data_change(id,
                        Block::Abbreviation(current_abbreviation.get_untracked()),
                        Block::Abbreviation(abbreviation.get_untracked().into()))
                    );
                // now set the new savepoint
                current_abbreviation.write().expansion = abbreviation.get_untracked().expansion;
            }
        />
        </div>
        <Accordion
            expand={expansion_config_expanded}
            expanded={Box::new(|| view! { <CogSvg/> }.into_any())}
            collapsed={Box::new(|| view! { <CogSvg/> }.into_any())}
        >
            <List>
                <Item align={Align::Left}>
                    <span class="font-light text-xs">"Expansion Language: "</span>
                    <input
                    prop:value=move || abbreviation.read().expansion_lang.clone()
                    class="text-sm"
                    placeholder="expansion-language"
                    autocomplete="false"
                    spellcheck="false"
                    id={format!("block-input-{id}-expansion_lang")}
                    on:input:target=move |ev| {
                        abbreviation.write().expansion_lang = ev.target().value();
                    }
                    on:change:target=move |ev| {
                        abbreviation.write().expansion_lang = ev.target().value();
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Abbreviation(current_abbreviation.get_untracked()),
                                Block::Abbreviation(abbreviation.get_untracked().into()))
                            );
                        // now set the new savepoint
                        current_abbreviation.write().expansion_lang = abbreviation.read_untracked().expansion_lang.clone();
                    }/>
                </Item>
            </List>
        </Accordion>
        </div>
    }
}

fn inner_correction_view(
    undo_stack: RwSignal<UnReStack>,
    correction: RwSignal<Correction>,
    focus_element: leptos::prelude::NodeRef<Textarea>,
    id: usize,
) -> impl IntoView {
    let current_correction = RwSignal::new(correction.get_untracked());

    let default_language = correction
        .read_untracked()
        .lang()
        .map_or("LANGUAGE".to_string(), std::string::ToString::to_string);

    let add_version = move |_| {
        let new_version = Version {
            hand: None,
            lang: default_language.clone(),
            content: String::default(),
        };
        correction.write().versions.push(new_version.clone());
        undo_stack.write().push_undo(UnReStep::new_data_change(
            id,
            Block::Correction(current_correction.get_untracked()),
            Block::Correction(correction.get_untracked().into()),
        ));
        // also push the change to the checkpoint
        current_correction.write().versions.push(new_version);
    };

    view! {
        <span
            class="font-light text-xs">
                "Correction with these versions:"
        </span>
        <For
            each=move || correction.get().versions.into_iter().enumerate()
            key=|dyn_v| dyn_v.0.clone()
            children = {move |dyn_v| {
                let memo_val = Memo::new(move |_| {
                    correction.read().versions.get(dyn_v.0).map_or(Version {
                        hand: None,
                        lang: String::default(),
                        content: String::default(),
                    }, |v| v.clone())
                });
                let config_expanded = signal(false);
                view!{
                    <div class="flex justify-between">
                    <span class="font-light text-xs">
                        "Version "{dyn_v.0}":"
                    </span>
                    <div>
                        <textarea
                        class="bg-orange-100 text-black font-mono"
                        id={format!("block-input-{id}-v-{}", dyn_v.0)}
                        node_ref=focus_element
                        prop:value=move || memo_val.read().content.clone()
                        autocomplete="false"
                        spellcheck="false"
                        rows=1
                        cols=TEXTAREA_DEFAULT_COLS
                        on:input:target=move |ev| {
                            if let Some(version_in_correction) = correction.write().versions.get_mut(dyn_v.0) {
                                version_in_correction.content = ev.target().value();
                            };
                        }
                        on:change:target=move |ev| {
                            let new_value = ev.target().value();
                            // change the value in correction
                            if let Some(version_in_correction) = correction.write().versions.get_mut(dyn_v.0) {
                                version_in_correction.content = new_value.clone();
                            };
                            undo_stack.write().push_undo(
                                UnReStep::new_data_change(id,
                                    Block::Correction(current_correction.get_untracked()),
                                    Block::Correction(correction.get_untracked().into()))
                                );
                            // now set the new savepoint
                            if let Some(version_in_correction) = current_correction.write().versions.get_mut(dyn_v.0) {
                                version_in_correction.content = new_value;
                            };
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
                                prop:value=move || memo_val.read().lang.clone()
                                class="text-sm"
                                placeholder="language"
                                autocomplete="false"
                                spellcheck="false"
                                id={format!("block-input-{id}-v-{}-lang", dyn_v.0)}
                                on:input:target=move |ev| {
                                    if let Some(version_in_correction) = correction.write().versions.get_mut(dyn_v.0) {
                                        version_in_correction.lang = ev.target().value();
                                    };
                                }
                                on:change:target=move |ev| {
                                    let new_lang = ev.target().value();
                                    // change the value in correction
                                    if let Some(version_in_correction) = correction.write().versions.get_mut(dyn_v.0) {
                                        version_in_correction.lang = new_lang.clone();
                                    };
                                    undo_stack.write().push_undo(
                                        UnReStep::new_data_change(id,
                                            Block::Correction(current_correction.get_untracked()),
                                            Block::Correction(correction.get_untracked().into()))
                                        );
                                    // now set the new savepoint
                                    if let Some(version_in_correction) = current_correction.write().versions.get_mut(dyn_v.0) {
                                        version_in_correction.lang = new_lang;
                                    };
                                }/>
                            </Item>
                            <Item align={Align::Left}>
                                <span class="font-light text-xs">"Hand: "</span>
                                <input
                                prop:value=move || memo_val.read().hand.clone()
                                class="text-sm"
                                placeholder="hand"
                                autocomplete="false"
                                spellcheck="false"
                                id={format!("block-input-{id}-v-{}-hand", dyn_v.0)}
                                on:input:target=move |ev| {
                                    // here we can allow the value to be unparsable, but we want to prevent
                                    // this if possible
                                    let x = ev.target().value();
                                    if let Some(version_in_correction) = correction.write().versions.get_mut(dyn_v.0) {
                                        version_in_correction.hand = if x.is_empty() {
                                                None
                                            } else {
                                                Some(x)
                                            };
                                    };
                                }
                                on:change:target=move |ev| {
                                    let x = ev.target().value();
                                    let new_hand = if x.is_empty() {
                                                None
                                            } else {
                                                Some(x)
                                            };
                                    if let Some(version_in_correction) = correction.write().versions.get_mut(dyn_v.0) {
                                        version_in_correction.hand = new_hand.clone();
                                    };
                                    undo_stack.write().push_undo(
                                        UnReStep::new_data_change(id,
                                            Block::Correction(current_correction.get_untracked()),
                                            Block::Correction(correction.get_untracked().into()))
                                        );
                                    // now set the new savepoint
                                    if let Some(version_in_correction) = current_correction.write().versions.get_mut(dyn_v.0) {
                                        version_in_correction.hand = new_hand;
                                    };
                                }/>
                            </Item>
                        </List>
                    </Accordion>
                    <button on:click=move |_| {
                        correction.write().versions.remove(dyn_v.0);
                        undo_stack.write().push_undo(
                            UnReStep::new_data_change(id,
                                Block::Correction(current_correction.get_untracked()),
                                Block::Correction(correction.get_untracked().into()))
                            );
                        // also push the change to the checkpoint
                        current_correction.write().versions.remove(dyn_v.0);
                    }><svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-4"><path stroke-linecap="round" stroke-linejoin="round" d="M12 9.75 14.25 12m0 0 2.25 2.25M14.25 12l2.25-2.25M14.25 12 12 14.25m-2.58 4.92-6.374-6.375a1.125 1.125 0 0 1 0-1.59L9.42 4.83c.21-.211.497-.33.795-.33H19.5a2.25 2.25 0 0 1 2.25 2.25v10.5a2.25 2.25 0 0 1-2.25 2.25h-9.284c-.298 0-.585-.119-.795-.33Z" /></svg></button>
                    </div>
                }}}
            />

            <br/>
            <button on:click=add_version><svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="size-6"><path stroke-linecap="round" stroke-linejoin="round" d="M12 9v6m3-3H9m12 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" /></svg></button>
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
        InnerBlock::Lacuna(lacuna) => inner_lacuna_view(undo_stack, lacuna, id).into_any(),
        InnerBlock::Uncertain(uncertain) => {
            inner_uncertain_view(undo_stack, uncertain, focus_element, id).into_any()
        }
        InnerBlock::Break(break_block) => inner_break_view(undo_stack, break_block, id).into_any(),
        InnerBlock::Anchor(anchor) => inner_anchor_view(undo_stack, anchor, id).into_any(),
        InnerBlock::Abbreviation(abbreviation) => {
            inner_abbreviation_view(undo_stack, abbreviation, focus_element, id).into_any()
        }
        InnerBlock::Correction(correction) => {
            inner_correction_view(undo_stack, correction, focus_element, id).into_any()
        }
        InnerBlock::Space(space) => inner_space_view(undo_stack, space, id).into_any(),
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
            InnerBlock::Space(x) => match other {
                Block::Space(y) => x.read_untracked() == *y,
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
    /// A part of text that is damaged but still legible
    Uncertain(RwSignal<Uncertain>),
    /// An expanded abbreviation
    Abbreviation(RwSignal<Abbreviation>),
    /// A bit of whitespace in the manuscript
    Space(RwSignal<Space>),
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
            Self::Space(x) => match new_block {
                Block::Space(new_space) => {
                    *x.write() = new_space;
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

    /// Create a new Block with this ones metadata, but change the content to be the new string
    fn clone_with_new_content(&self, new_content: String) -> InnerBlock {
        match self {
            InnerBlock::Break(_) => self.clone(),
            InnerBlock::Space(space) => InnerBlock::Space(RwSignal::new(Space {
                quantity: space.read_untracked().quantity,
                unit: space.read_untracked().unit,
            })),
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
                    versions: vec![Version {
                        lang: correction
                            .read_untracked()
                            .lang()
                            .clone()
                            .unwrap_or("LANGUAGE")
                            .to_string(),
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
                    surface_lang: abbreviation.read_untracked().surface_lang.clone(),
                    expansion_lang: abbreviation.read_untracked().expansion_lang.clone(),
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
            InnerBlock::Lacuna(_) => None,
            InnerBlock::Space(_) => None,
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
            InnerBlock::Lacuna(_) => None,
            InnerBlock::Space(_) => None,
            InnerBlock::Anchor(_) => None,
            InnerBlock::Uncertain(x) => Some(x.read_untracked().lang.clone()),
            InnerBlock::Correction(x) => x
                .read_untracked()
                .lang()
                .clone()
                .map(std::string::ToString::to_string),
            InnerBlock::Abbreviation(x) => Some(x.read_untracked().expansion_lang.clone()),
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
            InnerBlock::Space(x) => Block::Space(x.get_untracked()),
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
            Block::Space(x) => InnerBlock::Space(RwSignal::new(x)),
        }
    }
}

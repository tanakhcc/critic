//! Lower-third editor for a line that is selected

use critic_components::{
    editor::{blocks::EditorBlock, Editor},
    xmleditor::{XmlEditor, XmlState},
};
use critic_shared::ShowHelp;
use leptos::{either::Either, prelude::*};

/// The shortcuts available in the editor
const SHORTCUT_DESCRIPTIONS: &[(&str, &str, &str)] = &[
    (
        "s",
        "Save",
        "Save the current state of the editor to the server",
    ),
    ("z", "Undo", "Undo your last action"),
    ("r", "Redo", "Redo the action you just undid"),
    ("t", "Text", "Add a new block of text without markup"),
    (
        "a",
        "Abbreviation",
        "Turn the selection into an abbreviation",
    ),
    ("u", "Uncertain", "Mark the selection as uncertain"),
    ("l", "Lacuna", "Mark the selection as lacunous"),
    ("c", "Correction", "Mark the selection as corrected"),
    (
        "v",
        "Verse",
        "Delete the selection, putting a verse boundary in its place",
    ),
    (
        "<space>",
        "Space",
        "Delete the selection, marking intended whitespace",
    ),
    (
        "<enter>",
        "Enter",
        "Delete the selection, marking the end of a line or column",
    ),
    ("c", "Check", "XML only: check that XML is valid."),
];

/// HelpOverlay for the Editor
#[component]
fn HelpOverlay(active: RwSignal<ShowHelp>) -> impl IntoView {
    view! {
        <div
            on:click=move |_| { active.update(|a| a.set_off()) }
            // my tailwind is not compiling backdrop-blur-xs and I don't know why..
            class="z-10 absolute inset-0 w-full bg-slate-900/90 backdrop-blur-[8px] overflow-y-auto"
            class=("block", move || active.read().get())
            class=("hidden", move || !active.read().get())
        >
            <div class="absolute left-20 w-4/5 text-lg text-white">
                <p>
                    "This is the transcription editor. Copy a base text from another edition, then edit it here, marking up differences you find in the manuscript image."
                </p>
                <p>
                    "You can use the normal Editor, view an approximated render of what you have entered so far, or edit the XML directly. Remember that when you edit XML, you need to convert it to the normal editor before saving or publishing to make sure the data is correct."
                </p>
                <p>
                    "You can use these keyboard shortcuts: "
                    <span class="text-2xl">ctrl + alt +</span>"..."
                </p>
                <table class="table-fixed flex justify-around">
                    <tbody>
                        {SHORTCUT_DESCRIPTIONS
                            .iter()
                            .map(|(key, name, descr)| {
                                view! {
                                    <tr>
                                        <td class="text-2xl w-28">{*key}</td>
                                        <td class="text-xl w-36">{*name}</td>
                                        <td>{*descr}</td>
                                    </tr>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum EditorTabs {
    Block,
    Xml,
}

/// The Editor, containing both the raw block tab and the XmlEditor Tab
#[component]
pub fn EditorWithTabs(
    blocks: RwSignal<Vec<EditorBlock>>,
    default_language: String,
    on_save: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
    on_publish: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
) -> impl IntoView {
    let help_active: RwSignal<ShowHelp> = use_context().expect("Root mounts ShowHelp context");
    let tab_active = RwSignal::new(EditorTabs::Block);

    let xml_state = RwSignal::new(XmlState::Checked);

    view! {
        <div class="mx-16 my-4 flex flex-col h-full bg-slate-800 relative">
            <HelpOverlay active=help_active />
            <div class="text-red">
                <p>
                    {move || match xml_state.get() {
                        XmlState::Checked | XmlState::Unchecked => Either::Left(()),
                        XmlState::Err(e) => Either::Right(e),
                    }}
                </p>
            </div>
            <TabSwitcher xml_state=xml_state tab_active=tab_active />
            {move || {
                tab_active
                    .with(|tab| match tab {
                        EditorTabs::Block => {
                            let lang_cloned = default_language.clone();
                            Either::Left(
                                view! {
                                    <Editor
                                        blocks=blocks
                                        default_language=lang_cloned
                                        on_save=on_save
                                    />
                                },
                            )
                        }
                        EditorTabs::Xml => {
                            Either::Right(
                                view! {
                                    <XmlEditor
                                        blocks=blocks
                                        on_save=on_save
                                        xml_state=xml_state
                                        default_language=default_language.clone()
                                    />
                                },
                            )
                        }
                    })
            }}
        </div>
        <PublishButton
            xml_state=xml_state.read_only()
            on_publish=on_publish
            blocks=blocks.read_only()
        />
    }
}

/// Switches between the raw block tab and the XmlEditor Tab in the Editor.
#[component]
fn TabSwitcher(xml_state: RwSignal<XmlState>, tab_active: RwSignal<EditorTabs>) -> impl IntoView {
    view! {
        <div id="editor-tab-header" class="mb-4 p-2 pb-0 border-b border-slate-600">
            <button
                on:click=move |_| {
                    match xml_state.get() {
                        XmlState::Checked => {
                            tab_active.set(EditorTabs::Block);
                        }
                        XmlState::Err(_) => {}
                        XmlState::Unchecked => {
                            xml_state
                                .set(XmlState::Err("You need to check the XML first.".to_string()));
                        }
                    }
                }
                class="mx-2 mb-0 p-2 hover:bg-slate-500 rounded-t-lg"
                class=("bg-sky-600/30", move || tab_active.get() == EditorTabs::Block)
            >
                Editor
            </button>
            <button
                on:click=move |_| {
                    tab_active.set(EditorTabs::Xml);
                }
                class="mx-2 mb-0 p-2 hover:bg-slate-500 rounded-t-lg"
                class=("bg-sky-600/30", move || tab_active.get() == EditorTabs::Xml)
            >
                XML
            </button>
        </div>
    }
}

/// Button that handles publishing of the transcription
#[component]
fn PublishButton(
    xml_state: ReadSignal<XmlState>,
    on_publish: Action<Vec<EditorBlock>, Result<(), ServerFnError>>,
    blocks: ReadSignal<Vec<EditorBlock>>,
) -> impl IntoView {
    view! {
        <div class="flex justify-center w-full">
            {move || {
                xml_state
                    .with(|state| match state {
                        XmlState::Checked => {
                            Either::Left(
                                view! {
                                    <button
                                        class="w-96 text-2xl m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50 shadow-sm shadow-sky-600 hover:bg-slate-500"
                                        on:click=move |_| {
                                            on_publish.dispatch(blocks.get());
                                        }
                                    >
                                        "Publish this transcription"
                                    </button>
                                },
                            )
                        }
                        XmlState::Unchecked => {
                            Either::Right(
                                view! {
                                    <span class="w-96 text-2xl m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50">
                                        "Check your XML before publishing!"
                                    </span>
                                },
                            )
                        }
                        XmlState::Err(_) => {
                            Either::Right(
                                view! {
                                    <span class="w-96 text-2xl m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50">
                                        "Fix errors before publishing!"
                                    </span>
                                },
                            )
                        }
                    })
            }}
        </div>
    }
}

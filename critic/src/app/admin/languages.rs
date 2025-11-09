//! Adding Models and settings for retraining

// route paths
// /admin/languages
//      /:language

use critic_components::DEFAULT_BUTTON_CLASSES;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_params;
use serde::{Deserialize, Serialize};

use crate::app::shared::LanguageParams;

#[server]
async fn get_languages() -> Result<Vec<critic_shared::LanguageMetadata>, ServerFnError> {
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;
    critic_server::db::get_languages(&config.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn add_language(language: String) -> Result<(), ServerFnError> {
    if language.is_empty() {
        return Err(ServerFnError::new("Manuscript name must not be empty."));
    }
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;
    // after adding the new language, redirect to its own page
    critic_server::db::add_language_with_default_options(&config.db, &language)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    leptos_axum::redirect(&format!("/admin/languages/{language}"));
    Ok(())
}

#[component]
pub fn LanguageList() -> impl IntoView {
    let language_list = Resource::new(
        move || false,
        async move |_| {
            get_languages()
                .await
                .map_err(|e| ServerFnError::new(format!("Unable to get languages: {e}")))
        },
    );

    let new_language_open = RwSignal::new(false);
    let add_language_srvact = ServerAction::<AddLanguage>::new();
    let new_language_error = move || match add_language_srvact.value().get() {
        Some(Err(e)) => Some(e.to_string()),
        _ => None,
    };
    let new_language = RwSignal::new(String::default());

    view! {
        <div id="language-wrapper" class="h-full flex flex-row justify-start">
            // the left sidebar containing the different languages
            <div
                id="language-sidebar-wrapper"
                class="flex flex-col justify-start w-1/4 overflow-auto border-r-2 border-slate-600"
            >
                // the new-language-button and actual list
                <div id="new-language-error" class="bg-red-200">
                    {new_language_error}
                </div>
                <div
                    id="new-language-button"
                    class=(["flex", "flex-row", "justify-center"], move || !new_language_open.get())
                    class=("hidden", move || new_language_open.get())
                >
                    <button
                        class=DEFAULT_BUTTON_CLASSES
                        on:click=move |_| { new_language_open.update(|x| *x ^= true) }
                    >
                        "New Language"
                    </button>
                </div>
                <div
                    id="new-language-form"
                    class=("block", move || new_language_open.get())
                    class=("hidden", move || !new_language_open.get())
                    class="m-2 justify-start rounded-4xl border-2 border-slate-600 bg-slate-800 text-sm shadow-md shadow-sky-600"
                >
                    <form
                        class="flex flex-row justify-start"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            leptos::task::spawn_local(async move {
                                let _res = add_language(new_language.get_untracked()).await;
                                language_list.refetch();
                            });
                            new_language_open.update(|x| *x ^= true);
                        }
                    >
                        <input
                            class="w-0 grow border-0 ml-4 font-mono text-slate-400 m-2.5"
                            type="text"
                            name="msname"
                            bind:value=new_language
                        />
                        <button
                            type="submit"
                            disabled=move || new_language.get().is_empty()
                            class="min-w-20 text-md rounded-l-none rounded-2xl text-center font-bold"
                            class=(
                                ["bg-slate-600", "hover:bg-slate-500", "text-slate-50"],
                                move || !new_language.get().is_empty(),
                            )
                            class=(
                                ["bg-rose-400", "text-slate-800"],
                                move || new_language.get().is_empty(),
                            )
                        >
                            "Create"
                        </button>
                    </form>
                </div>
                <ErrorBoundary fallback=|errors| {
                    view! {
                        <div>
                            "Error: failed to get languages"
                            <ul>
                                {move || {
                                    errors
                                        .get()
                                        .into_iter()
                                        .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                        .collect::<Vec<_>>()
                                }}
                            </ul>
                        </div>
                    }
                }>
                    <Transition fallback=|| view! { <p>"Loading languages..."</p> }>
                        // list of languages
                        <div
                            id="language-list-wrapper"
                            class="flex flex-col justify-start h-0 grow"
                        >
                            <ul>
                                {move || {
                                    language_list
                                        .get()
                                        .map(|info_res| {
                                            info_res
                                                .map(|info: Vec<critic_shared::LanguageMetadata>| {
                                                    info.into_iter()
                                                        .map(|language| {
                                                            let language_params = use_params::<LanguageParams>();
                                                            let is_selected = || {
                                                                language_params
                                                                    .get()
                                                                    .is_ok_and(|param| {
                                                                        param.language.is_some_and(|param| &param == &language.name)
                                                                    })
                                                            };
                                                            view! {
                                                                <li class="flex">
                                                                    // keep query parameter if one is set
                                                                    <a
                                                                        href=format!("/admin/languages/{}", language.name.clone())
                                                                        class="w-0 grow my-2 bg-slate-600 p-2 text-center font-serif text-lg shadow-sm hover:bg-slate-500"
                                                                        class=(["shadow-sky-600"], !is_selected())
                                                                        class=(["shadow-slate-300", "text-sky-300"], is_selected())
                                                                    >
                                                                        {language.name.clone()}
                                                                    </a>
                                                                </li>
                                                            }
                                                        })
                                                        .collect_view()
                                                })
                                        })
                                }}
                            </ul>
                        </div>
                    </Transition>
                </ErrorBoundary>
            </div>

            // the information on the selected language
            <Outlet />
        </div>
    }
}

#[server]
pub async fn get_language_by_name(
    name: String,
) -> Result<critic_shared::LanguageMetadata, ServerFnError> {
    let config: std::sync::Arc<critic_server::config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    let res = critic_server::db::get_language_by_name(&config.db, &name).await;
    match res {
        Ok(Some(x)) => Ok(x),
        Ok(None) => Err(ServerFnError::new(format!(
            "Language {} does not exist in the db",
            name
        ))),
        Err(e @ critic_server::db::DBError::CannotGetLanguage(_)) => {
            Err(ServerFnError::new(e.to_string()))
        }
        Err(e) => {
            tracing::warn!("Failed loading language meta: {e}");
            Err(ServerFnError::new(e.to_string()))
        }
    }
}

/// Show the content for an individual language
#[component]
pub fn Language() -> impl IntoView {
    let params = use_params::<LanguageParams>();

    // get language from url
    let language = move || params.read().as_ref().ok().and_then(|x| x.language.clone());
    // now get language from the db
    let language_info = Resource::new(language, async |language_opt| {
        if let Some(lang) = language_opt {
            get_language_by_name(lang)
                .await
                .map_err(|e| ServerFnError::new(format!("Unable to get language information: {e}")))
        } else {
            Err(ServerFnError::new(
                "No language passed in the URL".to_string(),
            ))
        }
    });
    let recognition_models = OnceResource::new(async {
        super::get_models(critic_shared::ModelType::Recognition)
            .await
            .map_err(|e| ServerFnError::new(format!("Unable to get models: {e}")))
    });
    let segmentation_models = OnceResource::new(async {
        super::get_models(critic_shared::ModelType::Segmentation)
            .await
            .map_err(|e| ServerFnError::new(format!("Unable to get models: {e}")))
    });

    // is there a nicer way to prevent three levels of error unnesting here??
    view! {
        <Transition fallback=|| {
            view! { "Loading language information..." }
        }>
            {move || Suspend::new(async move {
                match language_info.await {
                    Err(e) => view! { <div>{e.to_string()}</div> }.into_any(),
                    Ok(language_info) => {
                        match segmentation_models.await {
                            Err(e) => view! { <div>{e.to_string()}</div> }.into_any(),
                            Ok(segmentation_models) => {
                                match recognition_models.await {
                                    Err(e) => view! { <div>{e.to_string()}</div> }.into_any(),
                                    Ok(recognition_models) => {
                                        view! {
                                            <div
                                                id="language-wrapper"
                                                class="h-full flex flex-col w-3/4 overflow-y-auto"
                                            >
                                                <LanguageMeta
                                                    meta=language_info
                                                    segmentation_models=segmentation_models
                                                    recognition_models=recognition_models
                                                />
                                            </div>
                                        }
                                            .into_any()
                                    }
                                }
                            }
                        }
                    }
                }
            })}
        </Transition>
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
struct UpdateLanguageModelData {
    language: String,
    segmentation_model_id: Option<i64>,
    recognition_model_id: Option<i64>,
}

/// Set the language retraining options for `language_id` to `retraining_opts`.
#[server]
async fn update_language_models(data: UpdateLanguageModelData) -> Result<(), ServerFnError> {
    use critic_server::auth::AuthSession;
    use critic_server::github::user_is_member;
    use leptos_axum::extract;

    let auth_session = match extract::<AuthSession>().await {
        Ok(x) => x,
        Err(e) => {
            let msg = format!("Failed to get AuthSession: {e}");
            tracing::warn!(msg);
            return Err(ServerFnError::new(msg));
        }
    };
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;

    let Some(user) = auth_session.user else {
        return Err(ServerFnError::new("No usersession available"));
    };
    match user_is_member(config.clone(), &user).await {
        Ok(true) => {}
        Ok(false) => {
            return Err(ServerFnError::new(
                "Unauthorized: Need to be Org member to update language retraining options.",
            ));
        }
        Err(e) => {
            tracing::warn!(
                "Unable to get github user membership for {}: {e}",
                user.username
            );
            return Err(ServerFnError::new(e.to_string()));
        }
    };
    if let Err(e) = critic_server::db::update_language(
        &config.db,
        &data.language,
        data.segmentation_model_id,
        data.recognition_model_id,
    )
    .await
    {
        tracing::warn!(
            "Failed to update language metadata for language with id {}",
            data.language,
        );
        return Err(ServerFnError::new(e.to_string()));
    };
    Ok(())
}

#[component]
fn LanguageMeta(
    meta: critic_shared::LanguageMetadata,
    segmentation_models: Vec<critic_shared::ModelMetadata>,
    recognition_models: Vec<critic_shared::ModelMetadata>,
) -> impl IntoView {
    let segmentation_model = RwSignal::new(meta.segmentation_model_id);
    let segmentation_model_saved = RwSignal::new(segmentation_model.get_untracked());
    let recognition_model = RwSignal::new(meta.recognition_model_id);
    let recognition_model_saved = RwSignal::new(recognition_model.get_untracked());

    let srvact = ServerAction::<UpdateLanguageModels>::new();

    view! {
        <div class="p-6 border-2 border-slate-500">
            // deliberately use the non-reactive old title here
            <h1 class="m-4 p-2 text-3xl text-center">
                "Language "<span class="font-bold">{meta.name.clone()}</span>
            </h1>
            <ActionForm action=srvact>
                <div class="flex justify-around flex-col">
                    <input type="hidden" name="data[language]" value=meta.name.clone() />
                    <div class="border border-slate-500 p-2 grid grid-cols-1">
                        <div class="grid grid-cols-2">
                            <label for="data[segmentation_model_id]">Segmentation model:</label>
                            <select
                                id="data[segmentation_model_id]"
                                name="data[segmentation_model_id]"
                                class="border border-slate-500 rounded-md"
                                // when no segmentation model is chosen (None), we want to write
                                // the empty string into the value here
                                prop:value=move || {
                                    segmentation_model
                                        .get()
                                        .map(|m| format!("{m}"))
                                        .unwrap_or_default()
                                }
                                on:change:target=move |evt| {
                                    segmentation_model
                                        .set(evt.target().value().parse::<i64>().ok());
                                }
                            >
                                <option value="" class="text-black">
                                    No automatic Segmentation
                                </option>
                                {move || {
                                    segmentation_models
                                        .iter()
                                        .map(|m| {
                                            view! {
                                                <option
                                                    class="text-black"
                                                    value=m.id
                                                    selected=segmentation_model.get_untracked() == Some(m.id)
                                                >
                                                    {m.name.clone()}
                                                </option>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </select>
                        </div>
                        <div class="grid grid-cols-2">
                            <label for="data[recognition_model_id]">Recognition model:</label>
                            <select
                                id="data[recognition_model_id]"
                                name="data[recognition_model_id]"
                                class="border border-slate-500 rounded-md"
                                prop:value=move || {
                                    recognition_model
                                        .get()
                                        .map(|m| format!("{m}"))
                                        .unwrap_or_default()
                                }
                                on:change:target=move |evt| {
                                    recognition_model.set(evt.target().value().parse::<i64>().ok());
                                }
                            >
                                <option value="">No automatic Recognition</option>
                                {move || {
                                    recognition_models
                                        .iter()
                                        .map(|m| {
                                            view! { <option value=m.id>{m.name.clone()}</option> }
                                        })
                                        .collect_view()
                                }}
                            </select>
                        </div>
                    </div>

                    <div class="flex justify-around mt-6">
                        <button
                            class=format!("w-2/5 {DEFAULT_BUTTON_CLASSES}")
                            type="button"
                            on:click=move |_| {
                                recognition_model.set(recognition_model_saved.get());
                                segmentation_model.set(segmentation_model_saved.get());
                            }
                        >
                            "Cancel"
                        </button>
                        <button
                            type="submit"
                            class=format!("w-2/5 {DEFAULT_BUTTON_CLASSES}")
                            // if the users saves an edit and does not reload the page, edits again
                            // and the clicks cancel, the last state already saved to the server
                            // would be overwritten here
                            on:click=move |_| {
                                recognition_model_saved.set(recognition_model.get());
                                segmentation_model_saved.set(segmentation_model.get());
                            }
                        >
                            Save changes
                        </button>
                    </div>
                </div>
            </ActionForm>
        </div>
    }
}

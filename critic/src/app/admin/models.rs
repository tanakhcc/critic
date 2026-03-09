//! Adding Models and settings for retraining

// route paths
// /admin/models
//      /recognition
//          /:id
//      /segmentation
//          /:id

use critic_components::filetransfer::TransferModel;
use critic_components::link_card::LinkCard;
use critic_components::DEFAULT_BUTTON_CLASSES;
use critic_shared::{ModelMetadata, ModelType, RetrainOptions};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::Outlet;
use leptos_router::hooks::use_params;
use serde::{Deserialize, Serialize};

use crate::app::shared::ModelParams;

#[server]
async fn add_model(modelname: String, model_type: ModelType) -> Result<i64, ServerFnError> {
    let config = use_context::<std::sync::Arc<critic_config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;
    // after adding the new manuscript, redirect to its own page
    let new_id = critic_db::add_model_with_default_options(&config.db, &modelname, model_type)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    leptos_axum::redirect(&format!("/admin/models/{model_type}/{new_id}"));
    Ok(new_id)
}

#[component]
pub fn ModelLanding() -> impl IntoView {
    view! {
        <div class="flex h-full flex-col">
            <div class="flex flex-row justify-center">
                <h1 class="p-10 text-6xl font-semibold">ML Model Administration</h1>
            </div>
            <div class="flex flex-row justify-center">
                <div class="grid w-3/4 grid-cols-3 gap-8">
                    <LinkCard header="Segmentation" link_to="/admin/models/segmentation">
                        <p class="ml-12 list-disc text-xl">Manage models for layout segmentation</p>
                    </LinkCard>
                    <LinkCard header="Recognition" link_to="/admin/models/recognition">
                        <p class="ml-12 list-disc text-xl">Manage models for text recognition</p>
                    </LinkCard>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Segmentation() -> impl IntoView {
    let model_list = Resource::new(
        || false,
        async move |_| {
            super::get_models(ModelType::Segmentation)
                .await
                .map_err(|e| ServerFnError::new(format!("Unable to get models: {e}")))
        },
    );

    view! {
        <div id="model-wrapper" class="h-full flex flex-row justify-start">
            // the left sidebar containing the different models
            <div
                id="model-sidebar-wrapper"
                class="flex flex-col justify-start w-1/4 overflow-auto border-r-2 border-slate-600"
            >
                <TransferModel
                    model_type=ModelType::Segmentation
                    on_new=move || model_list.refetch()
                />
                <ErrorBoundary fallback=|errors| {
                    view! {
                        <div>
                            "Error: failed to get manuscripts"
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
                    <Transition fallback=|| view! { <p>"Loading models..."</p> }>
                        // list of models
                        <div id="model-list-wrapper" class="flex flex-col justify-start h-0 grow">
                            <ul>
                                {move || {
                                    model_list
                                        .get()
                                        .map(|info_res| {
                                            info_res
                                                .map(|info: Vec<ModelMetadata>| {
                                                    info.into_iter()
                                                        .map(|model| {
                                                            let model_params = use_params::<ModelParams>();
                                                            let is_selected = move || {
                                                                model_params
                                                                    .get()
                                                                    .is_ok_and(|param| {
                                                                        param.id.is_some_and(|param| param == model.id)
                                                                    })
                                                            };
                                                            // we do not want to show MSS that the
                                                            // user did not search for
                                                            view! {
                                                                <li class="flex">
                                                                    // keep query parameter if one is set
                                                                    <a
                                                                        href=format!("/admin/models/segmentation/{}", model.id)
                                                                        class="w-0 grow my-2 bg-slate-600 p-2 text-center font-serif text-lg shadow-sm hover:bg-slate-500"
                                                                        class=(["shadow-sky-600"], !is_selected())
                                                                        class=(["shadow-slate-300", "text-sky-300"], is_selected())
                                                                    >
                                                                        {model.name.clone()}
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

            // the information on the selected model
            <Outlet />
        </div>
    }
}

#[server]
pub async fn get_model_by_id(
    id: i64,
    model_type: ModelType,
) -> Result<ModelMetadata, ServerFnError> {
    let config: std::sync::Arc<critic_config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    let res = critic_db::get_model_by_id(&config.db, id, model_type).await;
    match res {
        Ok(x) => Ok(x),
        Err(e @ critic_db::DBError::CannotGetModel(_)) => Err(ServerFnError::new(e.to_string())),
        Err(e) => {
            tracing::warn!("Failed loading model meta: {e}");
            Err(ServerFnError::new(e.to_string()))
        }
    }
}

/// Show the content for an individual model
#[component]
pub fn Model(model_type: ModelType) -> impl IntoView {
    let params = use_params::<ModelParams>();

    // get modelname from url
    let model_id_type = move || {
        params
            .read()
            .as_ref()
            .ok()
            .and_then(|x| Some((x.id.clone(), model_type)))
    };
    // now get model from the db
    let model_info = Resource::new(model_id_type, async |id_opt| {
        if let Some((Some(id), m_type)) = id_opt {
            get_model_by_id(id, m_type)
                .await
                .map_err(|e| ServerFnError::new(format!("Unable to get model information: {e}")))
        } else {
            Err(ServerFnError::new("No model passed in the URL".to_string()))
        }
    });

    view! {
        <Transition fallback=|| {
            view! { "Loading model information..." }
        }>
            {move || {
                model_info
                    .get()
                    .map(|info_res| match info_res {
                        Err(e) => Either::Left(view! { <div>{e.to_string()}</div> }),
                        Ok(info) => {
                            Either::Right(
                                view! {
                                    <div
                                        id="model-wrapper"
                                        class="h-full flex flex-col w-3/4 overflow-y-auto"
                                    >
                                        <ModelMeta meta=info />
                                    </div>
                                },
                            )
                        }
                    })
            }}
        </Transition>
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq)]
struct UpdateModelRetrainingOptsData {
    model_id: i64,
    model_type: ModelType,
    every_days: Option<u16>,
    keep_versions: Option<u16>,
}

/// Set the model retraining options for `model_id` to `retraining_opts`.
#[server]
async fn update_model_retraining_opts(
    data: UpdateModelRetrainingOptsData,
) -> Result<(), ServerFnError> {
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
    let config = use_context::<std::sync::Arc<critic_config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;

    let Some(user) = auth_session.user else {
        return Err(ServerFnError::new("No usersession available"));
    };
    match user_is_member(config.clone(), &user).await {
        Ok(true) => {}
        Ok(false) => {
            return Err(ServerFnError::new(
                "Unauthorized: Need to be Org member to update Model retraining options.",
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
    // change the model in the db
    let retraining_opts = if let Some(retrain_every_days) = data.every_days {
        Some(RetrainOptions {
            every_days: retrain_every_days,
            keep_versions: data.keep_versions,
        })
    } else {
        None
    };
    if let Err(e) =
        critic_db::update_model(&config.db, data.model_id, data.model_type, &retraining_opts).await
    {
        tracing::warn!(
            "Failed to update model metadata for model with id {}",
            data.model_id,
        );
        return Err(ServerFnError::new(e.to_string()));
    };
    Ok(())
}

#[component]
fn ModelMeta(meta: ModelMetadata) -> impl IntoView {
    let should_retrain = RwSignal::new(meta.retrain_options.is_some());
    let should_retrain_saved = RwSignal::new(should_retrain.get_untracked());

    let every_days_saved = RwSignal::new(
        meta.retrain_options
            .as_ref()
            .map(|r| r.every_days)
            .unwrap_or_default(),
    );
    let every_days_current_edit = if should_retrain.get_untracked() {
        RwSignal::new(Some(every_days_saved.get_untracked()))
    } else {
        RwSignal::new(None)
    };
    let keep_versions_saved = RwSignal::new(
        meta.retrain_options
            .as_ref()
            .map(|r| r.keep_versions)
            .unwrap_or_default(),
    );
    let keep_versions_current_edit = if should_retrain.get_untracked() {
        RwSignal::new(Some(keep_versions_saved.get_untracked()))
    } else {
        RwSignal::new(None)
    };

    let srvact = ServerAction::<UpdateModelRetrainingOpts>::new();

    view! {
        <div class="p-6 border-2 border-slate-500">
            // deliberately use the non-reactive old title here
            <h1 class="m-4 p-2 text-3xl text-center">
                "Model "<span class="font-bold">{meta.name.clone()}</span>
            </h1>
            <ActionForm action=srvact>
                <div class="flex justify-around flex-col">
                    <input type="hidden" name="data[model_id]" value=meta.id />
                    <div class="border border-slate-500 p-2 grid grid-cols-1">
                        <div class="grid grid-cols-2">
                            <label for="data[should_retrain]">Retrain:</label>
                            <input
                                id="data[should_retrain]"
                                name="data[should_retrain]"
                                class="border border-slate-500 rounded-md"
                                type="checkbox"
                                prop:checked=move || should_retrain.get()
                                on:click=move |_evt| {
                                    should_retrain.update(|sr| *sr = !*sr);
                                    if should_retrain.get() {
                                        every_days_current_edit.set(Some(every_days_saved.get()));
                                        keep_versions_current_edit
                                            .set(Some(keep_versions_saved.get()));
                                    } else {
                                        every_days_current_edit.set(None);
                                        keep_versions_current_edit.set(None);
                                    }
                                }
                            />
                        </div>

                        {move || {
                            if should_retrain.get() {
                                Either::Left(
                                    view! {
                                        <div class="grid grid-cols-2">
                                            <label for="data[every_days]">Retrain every n days:</label>
                                            <input
                                                id="data[every_days]"
                                                name="data[every_days]"
                                                class="border border-slate-500 rounded-md"
                                                prop:value=move || {
                                                    every_days_current_edit.get().unwrap_or_default()
                                                }
                                                autocomplete="false"
                                                spellcheck="false"
                                                placeholder="n"
                                                on:change:target=move |ev| {
                                                    *every_days_current_edit.write() = Some(
                                                        ev.target().value().parse::<u16>().unwrap_or(7),
                                                    );
                                                }
                                            />
                                        </div>
                                        <div class="grid grid-cols-2">
                                            <label for="data[keep_versions]">
                                                Keep the last n trained versions:
                                            </label>
                                            <input
                                                id="data[keep_versions]"
                                                name="data[keep_versions]"
                                                class="border border-slate-500 rounded-md"
                                                prop:value=move || {
                                                    keep_versions_current_edit
                                                        .get()
                                                        .unwrap_or_default()
                                                        .map(|v| format!("{v}"))
                                                        .unwrap_or_default()
                                                }
                                                autocomplete="false"
                                                spellcheck="false"
                                                placeholder="n"
                                                on:change:target=move |ev| {
                                                    *keep_versions_current_edit.write() = Some(
                                                        ev
                                                            .target()
                                                            .value()
                                                            .parse::<u16>()
                                                            .map(|x| Some(x))
                                                            .unwrap_or(None),
                                                    );
                                                }
                                            />
                                        </div>
                                    },
                                )
                            } else {
                                Either::Right(
                                    view! {
                                        <p>Do not Retrain.</p>
                                        <p>Keep all existing trained versions.</p>
                                    },
                                )
                            }
                        }}
                    </div>

                    <div class="flex justify-around mt-6">
                        <button
                            class=format!("w-2/5 {DEFAULT_BUTTON_CLASSES}")
                            type="button"
                            on:click=move |_| {
                                if should_retrain_saved.get() {
                                    should_retrain.set(true);
                                    every_days_current_edit.set(Some(every_days_saved.get()));
                                    keep_versions_current_edit.set(Some(keep_versions_saved.get()));
                                } else {
                                    should_retrain.set(false);
                                    every_days_current_edit.set(None);
                                    keep_versions_current_edit.set(None);
                                }
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
                                if should_retrain.get() {
                                    should_retrain_saved.set(true);
                                    every_days_saved
                                        .set(every_days_current_edit.get().unwrap_or_default());
                                    keep_versions_saved
                                        .set(keep_versions_current_edit.get().unwrap_or_default());
                                } else {
                                    should_retrain_saved.set(false);
                                }
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

#[component]
pub fn Recognition() -> impl IntoView {
    let model_list = Resource::new(
        || false,
        async move |_| {
            super::get_models(ModelType::Recognition)
                .await
                .map_err(|e| ServerFnError::new(format!("Unable to get models: {e}")))
        },
    );

    view! {
        <div id="model-wrapper" class="h-full flex flex-row justify-start">
            // the left sidebar containing the different models
            <div
                id="model-sidebar-wrapper"
                class="flex flex-col justify-start w-1/4 overflow-auto border-r-2 border-slate-600"
            >
                <TransferModel
                    model_type=ModelType::Recognition
                    on_new=move || model_list.refetch()
                />
                <ErrorBoundary fallback=|errors| {
                    view! {
                        <div>
                            "Error: failed to get manuscripts"
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
                    <Transition fallback=|| view! { <p>"Loading models..."</p> }>
                        // list of models
                        <div id="model-list-wrapper" class="flex flex-col justify-start h-0 grow">
                            <ul>
                                {move || {
                                    model_list
                                        .get()
                                        .map(|info_res| {
                                            info_res
                                                .map(|info: Vec<ModelMetadata>| {
                                                    info.into_iter()
                                                        .map(|model| {
                                                            let model_params = use_params::<ModelParams>();
                                                            let is_selected = move || {
                                                                model_params
                                                                    .get()
                                                                    .is_ok_and(|param| {
                                                                        param.id.is_some_and(|param| param == model.id)
                                                                    })
                                                            };
                                                            // we do not want to show MSS that the
                                                            // user did not search for
                                                            view! {
                                                                <li class="flex">
                                                                    // keep query parameter if one is set
                                                                    <a
                                                                        href=format!("/admin/models/recognition/{}", model.id)
                                                                        class="w-0 grow my-2 bg-slate-600 p-2 text-center font-serif text-lg shadow-sm hover:bg-slate-500"
                                                                        class=(["shadow-sky-600"], !is_selected())
                                                                        class=(["shadow-slate-300", "text-sky-300"], is_selected())
                                                                    >
                                                                        {model.name.clone()}
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

            // the information on the selected model
            <Outlet />
        </div>
    }
}

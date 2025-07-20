//! Adding manuscripts, page images and setting metadata

// route paths
// /admin/manuscripts
//      /:msname
//          /:pagename
// query params
// @msq=search-term-to-find-ms

use critic_components::filetransfer::TransferPage;
use critic_components::{TEXTAREA_DEFAULT_COLS, TEXTAREA_DEFAULT_ROWS};
use critic_shared::urls::{IMAGE_BASE_LOCATION, STATIC_BASE_URL};
use critic_shared::ManuscriptMeta;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::{Outlet, A};
use leptos_router::hooks::{query_signal, use_params};
use leptos_router::params::Params;

#[derive(Params, Clone, PartialEq)]
struct MsParams {
    msname: Option<String>,
}

#[derive(Params, Clone, PartialEq)]
struct PageParams {
    pagename: Option<String>,
}

#[server]
async fn get_manuscripts_by_name(
    msname: Option<String>,
) -> Result<Vec<critic_shared::ManuscriptMeta>, ServerFnError> {
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;
    critic_server::db::get_manuscripts_by_name(&config.db, msname)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn add_manuscript(msname: String) -> Result<(), ServerFnError> {
    let config = use_context::<std::sync::Arc<critic_server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;
    // after adding the new manuscript, redirect to its own page
    leptos_axum::redirect(&format!("/admin/manuscripts/{msname}"));
    critic_server::db::add_manuscript(&config.db, msname)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn ManuscriptList() -> impl IntoView {
    let (query, set_query) = query_signal::<String>("msq");

    // this can be toggled to force a reload for manuscripts
    let force_manuscript_reload = RwSignal::new(false);
    let manuscript_list = Resource::new(
        move || (query.get(), force_manuscript_reload),
        async |new_query| {
            get_manuscripts_by_name(new_query.0).await.map_err(|e| {
                ServerFnError::new(format!("Unable to get manuscript information: {e}"))
            })
        },
    );
    let ms_search_ref = NodeRef::new();

    let new_manuscript_open = RwSignal::new(false);

    let add_manuscript_srvact = ServerAction::<AddManuscript>::new();

    let new_manuscript_error = move || match add_manuscript_srvact.value().get() {
        Some(Err(e)) => Some(e.to_string()),
        _ => None,
    };

    let new_msname_ref = NodeRef::new();
    view! {
        <div id="ManuscriptList-wrapper" class="h-full flex flex-row justify-start">
        // the left sidebar containing the different manuscripts
        <div id="ms-sidebar-wrapper" class="flex flex-col justify-start w-1/4 overflow-auto bg-violet-100">
            // the search bar, new-manuscript-button and actual list
            <div id="new-manuscript-error" class="bg-red-200">
                {new_manuscript_error}
            </div>
            <div
                id="new-manuscript-button"
                class=(["flex", "flex-row", "justify-center"], move || !new_manuscript_open.get())
                class=("hidden", move || new_manuscript_open.get())
                >
                <button
                    class="bg-violet-300 hover:bg-violet-200 border rounded-md border-slate-500"
                    on:click=move |_| {
                        // toggle visibility for this button and the new-manuscript-form
                        new_manuscript_open.update(|x| *x ^= true)
                }>
                    "New Manuscript"
                </button>
            </div>
            <div
                id="new-manuscript-form"
                class=("block", move || new_manuscript_open.get())
                class=("hidden", move || !new_manuscript_open.get())
                >
                <form
                    class="flex justify-start"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        let new_msname = new_msname_ref.get().expect("input field exists");
                        leptos::task::spawn_local(async move { let _res = add_manuscript(new_msname.value()).await; });
                        // toggle visibility back to the button
                        new_manuscript_open.update(|x| *x ^= true);
                        // refetch the data, which now contains the new manuscript
                        manuscript_list.refetch();
                    }
                    >
                    // `title` matches the `title` argument to `add_todo`
                    <input
                        class="w-0 grow border border-slate-500 rounded-md"
                        node_ref=new_msname_ref type="text" name="msname"/>
                    <button
                        class="min-w-20 bg-violet-300 hover:bg-violet-200 border border-slate-500 rounded-md"
                        type="submit">Create</button>
                </form>
            </div>
            // container for the search line and button
            <div id="search-wrapper" class="flex justify-between">
                <input
                    class="w-0 grow border border-slate-500 rounded-md"
                    node_ref=ms_search_ref type="search" id="manuscript-search" name="msq" value=move || query.get()/>
                <button
                    class="min-w-16 bg-sky-300 hover:bg-sky-200 border rounded-md border-slate-500"
                    on:click=move |_| {
                    let current_value = ms_search_ref.get().expect("statically linked to the dom").value();
                    set_query.set(if current_value.is_empty() { None } else { Some(current_value) });
                }>"Search"</button>
            </div>

            <ErrorBoundary fallback=|errors| view!{
                <div>
                    "Error: failed to get manuscripts"
                    <ul>
                        {move || errors.get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                            .collect::<Vec<_>>()
                        }
                    </ul>
                </div>
            }>
                <Transition fallback=|| view!{ <p>"Loading manuscripts..."</p> }>
                    // list of manuscripts
                    <div id="ms-list-wrapper" class="flex flex-col justify-start h-0 grow">
                    <ul>
                        {move ||
                            manuscript_list.get().map(|info_res|
                                info_res.map(|info| {
                                info.into_iter().map(|ms|
                                        view!{
                                                <li class="border-t-2 border-violet-700 pt-2 pb-2">
                                                {
                                                // keep query parameter if one is set
                                                if let Some(query_name) = query.get() {
                                                    Either::Left(
                                                        view!{
                                                            <A href=format!("{}?msq={}", ms.title, query_name)>{ms.title}</A>
                                                        })
                                                } else {
                                                    Either::Right(
                                                        view!{
                                                            <A href=format!("{}", ms.title)>{ms.title}</A>
                                                        })
                                                }
                                                }
                                                </li>
                                        }
                                    ).collect_view()
                            }))
                        }
                    </ul>
                    </div>
                </Transition>
            </ErrorBoundary>
        </div>

        // the information on the selected manuscript
        <Outlet/>
        </div>
    }
}

#[server]
pub async fn get_manuscript_by_name(
    msname: String,
) -> Result<critic_shared::Manuscript, ServerFnError> {
    let config: std::sync::Arc<critic_server::config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    let res = critic_server::db::get_manuscript(&config.db, &msname).await;
    match res {
        Ok(x) => Ok(x),
        Err(e @ critic_server::db::DBError::ManuscriptDoesNotExist(_)) => {
            Err(ServerFnError::new(e.to_string()))
        }
        Err(e) => {
            tracing::warn!("Failed loading manuscript meta: {e}");
            Err(ServerFnError::new(e.to_string()))
        }
    }
}

/// Show the content for an individual manuscript
#[component]
pub fn Manuscript() -> impl IntoView {
    let params = use_params::<MsParams>();
    // get msname from url
    let msname = move || params.read().as_ref().ok().and_then(|x| x.msname.clone());
    // now get manuscript from the db
    let manuscript_info = Resource::new(msname, async |name_opt| {
        if let Some(name) = name_opt {
            get_manuscript_by_name(name).await.map_err(|e| {
                ServerFnError::new(format!("Unable to get manuscript information: {e}"))
            })
        } else {
            Err(ServerFnError::new(
                "No manuscript passed in the URL".to_string(),
            ))
        }
    });

    view! {
        <Transition fallback=|| view!{ "Loading manuscript information..." }>
            {move ||
                manuscript_info.get().map(|info_res|
                    match info_res {
                        Err(e) => {
                            Either::Left(view!{
                                <div>
                                    {e.to_string()}
                                </div>
                            })
                        }
                        Ok(info) => {
                            let show_page_upload = RwSignal::new(false);
                            let msname = info.meta.title.clone();
                            Either::Right(
                            view!{
                                <div id="Manuscript-wrapper" class="h-full flex flex-col w-3/4 overflow-y-auto">
                                <ManuscriptMeta meta=info.meta />
                                // container for the lower half of the screen
                                <div class="flex flex-row w-full h-0 grow bg-pink-300">
                                    // wrapper around the page upload form - this is show over the
                                    // entire page-list and page info part of the page
                                    <Show when=move || show_page_upload.get()
                                          fallback=|| {}>
                                        <div class="absolute inset-0 bg-stone-100/60 backdrop-blur-[4px]">
                                            <div class="relative inset-1/12 w-10/12">
                                            <div class="bg-violet-50">
                                                <TransferPage msname=msname.clone() />
                                            </div>
                                            <div class="flex justify-around">
                                            <button on:click=move |_| {
                                                    // close the floating page upload form
                                                    show_page_upload.update(|x| *x = false);
                                                    // refresh the page view by reloading pages
                                                    // from the server to reflect newly uploaded
                                                    // pages
                                                    manuscript_info.refetch();
                                            }>
                                                Done
                                            </button>
                                            </div>
                                            </div>
                                        </div>
                                    </Show>
                                <div id="manuscript-pageinfo-wrapper" class="flex justify-start">
                                    // container for the left half of the lower half
                                    <div id="manuscript-pagelist-wrapper" class="flex-col justify-start w-36 overflow-auto">
                                        <button on:click=move |_| {
                                            // show the page upload form
                                            show_page_upload.update(|x| *x ^= true);
                                        }>"Add Pages"</button>
                                        // list over all pages
                                        <ul>
                                            {
                                                info.pages.into_iter().map(|page| view!{
                                                    <li>
                                                        <A href={page.name.clone()}>{page.name}</A>
                                                        {
                                                            if let (Some(start), Some(end)) = (page.verse_start, page.verse_end) {
                                                                Some(view!{<p>{start} - {end}</p>})
                                                            } else {
                                                                None
                                                            }
                                                        }
                                                    </li>
                                                }).collect_view()
                                            }
                                        </ul>
                                    </div>

                                    </div>
                                    // the buttons and preview for the selected page if any
                                    <Outlet/>
                                </div>
                                </div>
                            })
                        }
                    }
                )
            }
        </Transition>
    }
}

/// Manuscript Meta Text Area - keeps track of a textarea field in `signal`
#[component]
pub fn MMetaTextArea(
    /// the name of this input field
    name: &'static str,
    /// this signal is updated when the input changes
    signal: RwSignal<Option<String>>,
    /// rendered inside the label
    children: Children,
) -> impl IntoView {
    view! {
          <div class="grid grid-cols-2 border border-b-0 border-slate-500 p-2">
              <label for={name}>{children()}</label>
              <textarea id={name} name={name} class="border border-slate-500 rounded-md"
                prop:value=move || signal.get().unwrap_or_default()
                autocomplete="false"
                spellcheck="false"
                rows=TEXTAREA_DEFAULT_ROWS
                cols=TEXTAREA_DEFAULT_COLS
                on:change:target=move |ev| {
                    let x = ev.target().value();
                    *signal.write() = (!x.is_empty()).then_some(x);
                }
              />
          </div>
    }
}

/// Manuscript Meta Text Area - keeps track of a textarea field in `signal`
#[component]
pub fn MMetaInput(
    /// the name of this input field
    name: &'static str,
    /// this signal is updated when the input changes
    signal: RwSignal<Option<String>>,
    /// rendered inside the label
    children: Children,
) -> impl IntoView {
    view! {
          <div class="grid grid-cols-2 border border-b-0 border-slate-500 p-2">
              <label for={name}>{children()}</label>
              <input id={name} name={name} class="border border-slate-500 rounded-md"
                prop:value=move || signal.get().unwrap_or_default()
                autocomplete="false"
                spellcheck="false"
                on:change:target=move |ev| {
                    let x = ev.target().value();
                    *signal.write() = (!x.is_empty()).then_some(x);
                }
              />
          </div>
    }
}

/// TODO: correctly rename file directory
#[server]
async fn update_ms_metadata(data: ManuscriptMeta, old_title: String) -> Result<(), ServerFnError> {
    use critic_server::auth::AuthSession;
    use critic_server::gitlab::{get_user_role, GitlabUserRole};
    use critic_shared::urls::IMAGE_BASE_LOCATION;
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

    if let Some(user) = auth_session.user {
        let user_role = match get_user_role(config.clone(), &user).await {
            Ok(x) => x,
            Err(e) => {
                tracing::warn!("Unable to get the user role for {}: {e}", user.username);
                return Err(ServerFnError::new(e.to_string()));
            }
        };
        if user_role < GitlabUserRole::Maintainer {
            Err(ServerFnError::new(
                "Unauthorized: Need to be Maintainer to update MS metadata.",
            ))
        } else {
            // change the MS in the db
            if let Err(e) = critic_server::db::update_ms_meta(&config.db, &data).await {
                tracing::warn!(
                    "Failed to update manuscript metadata for ms with id {}",
                    data.id
                );
                return Err(ServerFnError::new(e.to_string()));
            };
            // rename the image directory for the MS if it was renamed
            if data.title != old_title {
                let base_path = format!("{}{IMAGE_BASE_LOCATION}", &config.data_directory);
                let old_path = format!("{base_path}/{old_title}");
                let new_path = format!("{base_path}/{}", data.title);
                if let Err(e) = std::fs::rename(&old_path, &new_path) {
                    // TODO - this raises errors when renaming MSs without pages because then the
                    // directory does not exist
                    // get pages first, and only raise this error when no page exists
                    tracing::warn!(
                        "Failed to rename {old_path} to {new_path} while upating ms metadata: {e}."
                    );
                };
                tracing::info!(
                    "User {} renamed MS {} to {}.",
                    user.username,
                    old_title,
                    data.title
                );
                // this is not quite enough - the MS will keep its wrong name in the left-hand
                // sidebar
                // But I don't really know how to change that behavior.
                leptos_axum::redirect(&format!("/admin/manuscripts/{}", data.title));
            };
            Ok(())
        }
    } else {
        Err(ServerFnError::new("No usersession available"))
    }
}

/// Show meta-information for an individual manuscript
#[component]
fn ManuscriptMeta(meta: critic_shared::ManuscriptMeta) -> impl IntoView {
    let institution = RwSignal::new(meta.institution.clone());
    let collection = RwSignal::new(meta.collection.clone());
    let hand_desc = RwSignal::new(meta.hand_desc.clone());
    let script_desc = RwSignal::new(meta.script_desc.clone());
    let new_name = RwSignal::new(meta.title.clone());

    let srvact = ServerAction::<UpdateMsMetadata>::new();

    view! {
      <div class="p-6 border-2 border-slate-500">
          // deliberately use the non-reactive old title here
        <h1 class="text-2xl m-4 bg-slate-300 p-2 border border-slate-500 rounded-md">"Manuscript "<span class="font-bold">{meta.title.clone()}</span></h1>
        <ActionForm
            action=srvact
            >
            <div
            class="flex justify-around flex-col"
            >
            <input type="hidden" name="data[id]" value={meta.id}/>
            <input type="hidden" name="old_title" value={meta.title.clone()}/>
            <MMetaInput name="data[institution]" signal=institution>Holding institution:</MMetaInput>
            <MMetaInput name="data[collection]" signal=collection>Collection:</MMetaInput>
            <MMetaTextArea name="data[hand_desc]" signal=hand_desc>Scribal hands in use:</MMetaTextArea>
            <MMetaTextArea name="data[script_desc]" signal=script_desc>Scripts in use:</MMetaTextArea>
            <details class="col-span-2 border border-slate-500 p-2">
                <summary>Rename this manuscript</summary>
                <div class="border border-slate-500 bg-red-100 mb-2">
                    <div class="p-4 pt-2 pb-2">
                    <p>
                        Warning! Renaming a manuscript will change its permalinks. Other users may find their links to this page breaking. Ideally, you only want to rename manuscripts right after creating them.
                    </p>
                </div>
                </div>
                <div class="grid grid-cols-2">
                    <label for="data[title]">New name</label>
                    <input id="data[title]" name="data[title]" class="border border-slate-500 rounded-md"
                        prop:value=move || new_name.get()
                        autocomplete="false"
                        spellcheck="false"
                        on:change:target=move |ev| {
                            *new_name.write() = ev.target().value();
                        }
                    />
            </div>
          </details>
          <div class="flex justify-around mt-6">
          <button class="bg-slate-100 hover:bg-slate-50 w-2/5 border rounded-xl"
          type="button"
          on:click=move |_| {
              // reset the signals
              *institution.write() = meta.institution.clone();
              *collection.write() = meta.collection.clone();
              *hand_desc.write() = meta.hand_desc.clone();
              *script_desc.write() = meta.script_desc.clone();
              *new_name.write() = meta.title.clone();
          }
          >Cancel</button>
          <button type="submit" class="bg-slate-300 hover:bg-slate-200 w-2/5 border rounded-xl"
          >Save changes</button>
          </div>
          </div>
        </ActionForm>
      </div>
    }
}

#[derive(Debug)]
struct EmptyError {}
impl core::fmt::Display for EmptyError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "An unspecified error occured.")
    }
}
impl std::error::Error for EmptyError {}

/// show information for a complete page
#[component]
pub fn Page() -> impl IntoView {
    let ms_params = use_params::<MsParams>();
    let page_params = use_params::<PageParams>();

    view! {
        <ErrorBoundary fallback=|_errors| view!{"Failed to get manuscriptname and page name from the url."}>
        {move ||
            if let (Some(msname), Some(pagename)) = (ms_params.get().map(|p| p.msname).unwrap_or(None), page_params.get().map(|p| p.pagename).unwrap_or(None)) {
                let image_base = format!("{STATIC_BASE_URL}{IMAGE_BASE_LOCATION}/{msname}/{pagename}");
                Ok(
                    view!{
                        <div class="flex w-0 flex-col grow justify-start">
                            <h2 class="text-xl m-2 bg-slate-300 p-1 border border-slate-500 rounded-md">"Page "<span class="font-bold">{pagename.clone()}</span></h2>
                            // container for the top line of this page
                            <div class="flex justify-start flex-row">
                                <a class="w-2/8 p-1 rounded-xl border bg-slate-300 hover:bg-slate-200" href={format!("/index/{msname}/{pagename}")}>"Index this page"</a>
                                <div class="flex flex-col w-2/8 p-1">
                                    <a class="rounded-xl border bg-slate-300 hover:bg-slate-200" href={format!("{image_base}/original.webp")} target="_blank">"Original Image"</a>
                                    <a class="rounded-xl border bg-slate-300 hover:bg-slate-200" href={format!("{image_base}/original.webp")} download=format!("{msname}_{pagename}.webp")>"Download Original Image"</a>
                                </div>
                                // simple form to change the name or image for this page
                                <button class="w-2/8 p-1 rounded-xl border bg-slate-300 hover:bg-slate-200">"Edit - TODO"</button>
                            </div>
                            // image preview for this page in the right hand side
                            <img alt={format!("Preview for {msname} - {pagename}")} src={format!("{image_base}/preview.webp")}/>
                        </div>
                    }
                )
            } else {
                Err(EmptyError{})
            }
        }
        </ErrorBoundary>
    }
}

#[component]
pub fn ManuscriptLanding() -> impl IntoView {
    view! {
        <p>
            "Select a manuscript from the left hand side to view or edit."
        </p>
    }
}

#[component]
pub fn PageLanding() -> impl IntoView {
    view! {
        <p>
            "Select a page from the left hand side to view or edit."
        </p>
    }
}

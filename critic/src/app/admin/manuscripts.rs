//! Adding manuscripts, page images and setting metadata

// route paths
// /admin/manuscripts
//      /:msname
//          /:pagename
// query params
// @msq=search-term-to-find-ms

use critic_components::filetransfer::TransferPage;
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
        <div id="ManuscriptList-wrapper" class="flex justify-start">
        // the left sidebar containing the different manuscripts
        <div id="ms-sidebar-wrapper" class="flex flex-col justify-start w-1/4">
            // the search bar, new-manuscript-button and actual list
            <div id="new-manuscript-error" class="bg-red-200">
                {new_manuscript_error}
            </div>
            <div
                id="new-manuscript-button"
                class="bg-blue-200"
                class=("block", move || new_manuscript_open.get() == false)
                class=("hidden", move || new_manuscript_open.get() == true)
                >
                <button
                    on:click=move |_| {
                        // toggle visibility for this button and the new-manuscript-form
                        new_manuscript_open.update(|x| *x ^= true)
                }>
                    "New Manuscript"
                </button>
            </div>
            <div
                id="new-manuscript-form"
                class="bg-blue-200"
                class=("block", move || new_manuscript_open.get() == true)
                class=("hidden", move || new_manuscript_open.get() == false)
                >
                <form
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
                    <input node_ref=new_msname_ref type="text" name="msname"/>
                    <input type="submit" value="Create Manuscript"/>
                </form>
            </div>
            // container for the search line and button
            <div id="search-wrapper" class="flex justify-between bg-sky-500">
                <input node_ref=ms_search_ref type="search" id="manuscript-search" name="msq" value=move || query.get()/>
                <button on:click=move |_| {
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
                    <div id="ms-list-wrapper" class="flex flex-col justify-start bg-emerald-500">
                    <ul>
                        {move ||
                            manuscript_list.get().map(|info_res|
                                info_res.map(|info| {
                                info.into_iter().map(|ms|
                                    // keep query parameter if one is set
                                    if let Some(query_name) = query.get() {
                                        Either::Left(
                                            view!{
                                                <li>
                                                <A href=format!("{}?msq={}", ms.title, query_name)>{ms.title}</A>
                                                </li>
                                            }
                                        )
                                    } else {
                                        Either::Right(
                                            view!{
                                                <li>
                                                <A href=format!("{}", ms.title)>{ms.title}</A>
                                                </li>
                                            }
                                        )
                                    }).collect_view()
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
    let res = critic_server::db::get_manuscript(&config.db, msname).await;
    match res {
        Ok(x) => Ok(x),
        Err(e @ critic_server::db::DBError::ManuscriptDoesNotExist(_)) => {
            return Err(ServerFnError::new(e.to_string()));
        }
        Err(e) => {
            tracing::warn!("Failed loading manuscript meta: {e}");
            return Err(ServerFnError::new(e.to_string()));
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

    return view! {
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
                                <div id="Manuscript-wrapper" class="h-full flex flex-col justify-between w-3/4">
                                <ManuscriptMeta meta=info.meta/>
                                // container for the lower half of the screen
                                <div class="relative h-full bg-pink-300">
                                    // wrapper around the page upload form - this is show over the
                                    // entire page-list and page info part of the page
                                    <Show when=move || show_page_upload.get() == true
                                          fallback=|| view!{}>
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
                                    <div id="manuscript-pagelist-wrapper" class="flex flex-col justify-start w-36">
                                        // TODO:
                                        // - create api endpoint to accept the uploads and create
                                        //   the pages
                                        // - reload this site to view new pages
                                        <button on:click=move |_| {
                                            // show the page upload form
                                            show_page_upload.update(|x| *x ^= true);
                                        }>"Add Pages"</button>
                                        // list over all pages
                                        <ul>
                                            {
                                                info.pages.into_iter().map(|page| view!{
                                                    <li>
                                                        <A href={page.name}>page.name</A>
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

                                    // the buttons and preview for the selected page if any
                                    <Outlet/>
                                </div>
                                </div>
                                </div>
                            })
                        }
                    }
                )
            }
        </Transition>
    };
}

/// Show meta-information for an individual manuscript
#[component]
pub fn ManuscriptMeta(meta: critic_shared::ManuscriptMeta) -> impl IntoView {
    view! {
        <div class="bg-amber-400">
            <h2>{format!("Manuscript Information for manuscript {}", meta.title)}</h2>
            <p>
                Hier steht irgend nen content und so.
            </p>
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
        {
            if let (Some(msname), Some(pagename)) = (ms_params.get().map(|p| p.msname).unwrap_or(None), page_params.get().map(|p| p.pagename).unwrap_or(None)) {
                Ok(
                    view!{
                        <div class="flex justify-start flex-row">
                        // container for the top line of this page
                        <div class="flex justify-start flex-row">
                            <a href={format!("/index/{msname}/{pagename}")}>"Index"</a>
                            // simple form to change the name or image for this page
                            <button>"Edit image or name"</button>
                        </div>
                        // image preview for this page in the right hand side
                        <img/>
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

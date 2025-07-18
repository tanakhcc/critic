//! Adding manuscripts, page images and setting metadata

// route paths
// /admin/manuscripts
//      /:msname
//          /:pagename
// query params
// @msq=search-term-to-find-ms

use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::components::{Outlet, A};
use leptos_router::hooks::{query_signal, use_params, use_query};
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
    msname: String,
) -> Result<Vec<crate::shared::ManuscriptMeta>, ServerFnError> {
    let config = use_context::<std::sync::Arc<crate::server::config::Config>>()
        .ok_or(ServerFnError::new("Unable to get config from context"))?;
    crate::server::db::get_manuscripts_by_name(&config.db, msname)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
pub fn ManuscriptList() -> impl IntoView {
    let (query, set_query) = query_signal::<String>("msq");

    let manuscript_list = Resource::new(
        move || query.get(),
        async |new_query| {
            if let Some(query_name) = new_query {
                get_manuscripts_by_name(query_name).await.map_err(|e| {
                    ServerFnError::new(format!("Unable to get manuscript information: {e}"))
                })
            } else {
                Err(ServerFnError::new(
                    "Failed to get query parameter from the url",
                ))
            }
        },
    );
    let ms_search_ref = NodeRef::new();

    view! {
        <div class="flex justify-start">
        <div class="flex flex-col justify-start w-48">
            // the search bar, new-manuscript-button and actual list
            <button>"New Manuscript"</button>
            // container for the search line and button
            <div class="flex justify-between">
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
                    <ul>
                        {move ||
                            manuscript_list.get().map(|info_res|
                                info_res.map(|info| {
                                info.into_iter().map(|ms|
                                    // keep query parameter if one is set
                                    if let Some(query_name) = query.get() {
                                        Either::Left(
                                            view!{
                                                <A href=format!("{}?msq={}", ms.title, query_name)>ms.title</A>
                                            }
                                        )
                                    } else {
                                        Either::Right(
                                            view!{
                                                <A href={ms.title}>ms.title</A>
                                            }
                                        )
                                    }).collect_view()
                            }))
                        }
                    </ul>
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
) -> Result<crate::shared::Manuscript, ServerFnError> {
    let config: std::sync::Arc<crate::server::config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    let res = crate::server::db::get_manuscript(&config.db, msname).await;
    match res {
        Ok(x) => Ok(x),
        Err(e @ crate::server::db::DBError::ManuscriptDoesNotExist(_)) => {
            return Err(ServerFnError::new(e.to_string()));
        }
        Err(e) => {
            tracing::warn!("Failed loading manuscript meta: {e}");
            return Err(ServerFnError::new(e.to_string()));
        }
    }
}

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
        <div class="flex justify-start flex-col">
        // the meta-information for this manuscript
        <ErrorBoundary fallback=|errors| view!{
            <div>
                "Error: failed to get manuscript information"
                <ul>
                    {move || errors.get()
                        .into_iter()
                        .map(|(_, e)| view! { <li>{e.to_string()}</li>})
                        .collect::<Vec<_>>()
                    }
                </ul>
            </div>
        }>
        <Transition fallback=|| view!{ "Loading manuscript information..." }>
            {
                manuscript_info.get().map(|info_res|
                    info_res.map(|info|
                view!{
                    <ManuscriptMeta meta=info.meta/>
                    // container for the lower half of the screen
                    <div class="flex justify-start">
                        // container for the left half of the lower half
                        <div class="flex flex-col justify-start w-36">
                        // TODO: simple menu to upload n files, their filename without extension will be used
                        // as page name
                        <button>"Add Pages"</button>
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
                            // scrollable
                            // show name, contain <A> to open /admin/manuscripts/<msname>/<pagename>
                        </ul>
                        </div>
                        // the buttons and preview for the selected page
                        <Outlet/>
                    </div>
                }))
            }
        </Transition>
        </ErrorBoundary>
        </div>
    }
}

#[component]
pub fn ManuscriptMeta(meta: crate::shared::ManuscriptMeta) -> impl IntoView {
    view! {
        // TODO: should probably be a <Form/>
        <h2>"Manuscript Information for manuscript"{meta.title}</h2>
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

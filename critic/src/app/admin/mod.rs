//! All the different admin pages

use leptos::prelude::*;
use leptos_router::components::{ParentRoute, Route};
use leptos_router::path;

use crate::app::TopLevelPosition;

mod manuscripts;

#[component]
pub fn AdminLanding() -> impl IntoView {
    let set_top_level_pos =
        use_context::<WriteSignal<TopLevelPosition>>().expect("App provides TopLevelPosition");
    *set_top_level_pos.write() = TopLevelPosition::Admin;

    view! {
        <div class="flex h-full flex-col">
            <div class="flex flex-row justify-center">
                <h1 class="p-10 text-6xl font-semibold">Critic Project Administration</h1>
            </div>
            <div class="flex flex-row justify-center">
                <div class="grid w-3/4 grid-cols-3 gap-8">
                    <a
                        href="/admin/manuscripts"
                        class="rounded-4xl border-2 border-sky-600 bg-slate-700 p-8 shadow-lg shadow-sky-600 hover:bg-slate-600 hover:shadow-xl"
                    >
                        <div class="flex flex-row justify-start">
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke-width="1.5"
                                stroke="currentColor"
                                class="size-14"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z"
                                />
                            </svg>
                            <h2 class="mb-4 text-4xl font-bold mt-3 ml-2">Manuscripts</h2>
                        </div>
                        <ul class="list-disc text-xl ml-12">
                            <li>Edit and add manuscripts</li>
                            <li>Upload panuscript pages</li>
                        </ul>
                    </a>
                    <a
                        href="/admin/versification"
                        class="rounded-4xl border-2 border-sky-600 bg-slate-700 p-8 shadow-lg shadow-sky-600 hover:bg-slate-600 hover:shadow-xl"
                    >
                        <div class="flex flex-row justify-start">
                            <svg
                                xmlns="http://www.w3.org/2000/svg"
                                fill="none"
                                viewBox="0 0 24 24"
                                stroke-width="1.5"
                                stroke="currentColor"
                                class="size-14"
                            >
                                <path
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    d="M7.5 3.75H6A2.25 2.25 0 0 0 3.75 6v1.5M16.5 3.75H18A2.25 2.25 0 0 1 20.25 6v1.5m0 9V18A2.25 2.25 0 0 1 18 20.25h-1.5m-9 0H6A2.25 2.25 0 0 1 3.75 18v-1.5M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                                />
                            </svg>
                            <h2 class="mt-3 mb-4 ml-2 text-4xl font-bold">Versification</h2>
                        </div>
                        <p class="ml-12 list-disc text-xl">Manage Versification Schemes</p>
                    </a>
                </div>
            </div>
        </div>
    }
}

#[component(transparent)]
pub fn AdminRouter() -> impl MatchNestedRoutes + Clone {
    let set_top_level_pos =
        use_context::<WriteSignal<TopLevelPosition>>().expect("App provides TopLevelPosition");
    *set_top_level_pos.write() = TopLevelPosition::Admin;

    view! {
        <ParentRoute path=path!("manuscripts") view=manuscripts::ManuscriptList>
            <ParentRoute path=path!(":msname") view=manuscripts::Manuscript>
                <Route path=path!(":pagename") view=manuscripts::Page />
                <Route path=path!("") view=manuscripts::PageLanding />
            </ParentRoute>
            <Route path=path!("") view=manuscripts::ManuscriptLanding />
        </ParentRoute>
    }
    .into_inner()
}

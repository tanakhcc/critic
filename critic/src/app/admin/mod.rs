//! All the different admin pages

use critic_components::link_card::LinkCard;
use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route};
use leptos_router::path;

use crate::app::TopLevelPosition;

mod manuscripts;
mod models;

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
                    <LinkCard header="Manuscripts" link_to="/admin/manuscripts">
                        <ul class="list-disc text-xl ml-12">
                            <li>Edit and upload manuscripts</li>
                            <li>Upload manuscript pages</li>
                        </ul>
                    </LinkCard>
                    <LinkCard header="Versification" link_to="/admin/versification">
                        <p class="ml-12 list-disc text-xl">Manage Versification Schemes</p>
                    </LinkCard>
                    <LinkCard header="ML Models" link_to="/admin/models">
                        <ul class="list-disc text-xl ml-12">
                            <li>Upload models for segmentation and recognition</li>
                            <li>Define model retraining intervals</li>
                        </ul>
                    </LinkCard>
                </div>
            </div>
        </div>
    }
}

#[component(transparent)]
pub fn AdminRouter() -> impl MatchNestedRoutes + Copy {
    let set_top_level_pos =
        use_context::<WriteSignal<TopLevelPosition>>().expect("App provides TopLevelPosition");
    *set_top_level_pos.write() = TopLevelPosition::Admin;

    view! {
        <ParentRoute
            path=path!("admin")
            view=|| {
                view! { <Outlet /> }
            }
        >
            <Route path=path!("") view=AdminLanding />
            <ParentRoute path=path!("manuscripts") view=manuscripts::ManuscriptList>
                <ParentRoute path=path!(":msname") view=manuscripts::Manuscript>
                    <Route path=path!(":pagename") view=manuscripts::Page />
                    <Route path=path!("") view=manuscripts::PageLanding />
                </ParentRoute>
                <Route path=path!("") view=manuscripts::ManuscriptLanding />
            </ParentRoute>
            <Route path=path!("models") view=models::ModelLanding />
            <ParentRoute path=path!("models/recognition") view=models::Recognition>
                <Route path=path!(":id") view=models::Model/>
                <Route path=path!("") view={|| view!{<p>Upload a model from the left hand side or upload a .mlmodel file.</p>}}/>
            </ParentRoute>
            <ParentRoute path=path!("models/segmentation") view=models::Segmentation>
                <Route path=path!(":id") view=models::Model/>
                <Route path=path!("") view={|| view!{<p>Upload a model from the left hand side or upload a .mlmodel file.</p>}}/>
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

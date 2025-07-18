//! All the different admin pages

use leptos::prelude::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes, A};
use leptos_router::path;
use leptos_router::MatchNestedRoutes;

mod manuscripts;

#[component]
pub fn AdminLanding() -> impl IntoView {
    view! {
        "This is the admin landing page"
        <A href="manuscripts">"Add or edit manuscripts"</A>
    }
}

#[component(transparent)]
pub fn AdminRouter() -> impl MatchNestedRoutes + Clone {
    view! {
        <ParentRoute path=path!("manuscripts") view=manuscripts::ManuscriptList>
            <Route path=path!("") view=manuscripts::ManuscriptLanding/>
            <ParentRoute path=path!(":msname") view=manuscripts::Manuscript>
                <Route path=path!("") view=manuscripts::PageLanding/>
                <Route path=path!(":pagename") view=manuscripts::Page/>
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

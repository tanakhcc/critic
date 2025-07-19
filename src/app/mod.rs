use leptos::{ev::keydown, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    path, StaticSegment,
};

mod editor;
use editor::Editor;
use leptos_use::{use_document, use_event_listener};

mod accordion;
mod admin;
mod filetransfer;
mod icons;

/// This provides context through the entire app. When ShowHelp(true) is present, some components
/// show a help-text.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ShowHelp(bool);
impl ShowHelp {
    pub(crate) fn toggle(&mut self) {
        self.0 ^= true
    }
    pub(crate) fn set_off(&mut self) {
        self.0 = false
    }
}
impl From<ShowHelp> for bool {
    fn from(value: ShowHelp) -> Self {
        value.0
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let help_active = RwSignal::new(ShowHelp(false));
    // event listener to intercept keycommands for the help menu
    let _cleanup = use_event_listener(use_document(), keydown, move |evt| {
        // <ctrl>-<alt>-H - Help
        if evt.alt_key() && evt.ctrl_key() && evt.key_code() == 72 {
            // toggle on/off help overlay
            help_active.update(|a| a.toggle())
        // <esc> - close Help if it is open
        } else if evt.key_code() == 27 {
            // turn off the overlay if it is currently on
            help_active.update(|a| a.set_off())
        }
    });
    provide_context(help_active);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/critic.css"/>

        // sets the document title
        <Title text="critic - textual criticism"/>

        // Router
        <Router>
            <nav>
                <a href="/get-started">"Get Started"</a>
                <a href="/admin">"Project Administration"</a>
                <a href="/transcribe">"Transcribe"</a>
                <a href="/reconcile">"Reconcile"</a>
                <span on:click=move |_| {
                    help_active.update(|a| a.toggle())
                }>
                    "Press ctrl+alt+h anywhere to get help."
                </span>
            </nav>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <ParentRoute path=path!("admin") view=|| {view!{ <div><Outlet/></div>}}>
                        <Route path=path!("") view=admin::AdminLanding/>
                        <admin::AdminRouter/>
                    </ParentRoute>
                    <Route path=path!("/editor") view=|| {
                        view! {
                            <Editor default_language="hbo-Hebr".to_string()/>
                        }.into_view()
                    }/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}

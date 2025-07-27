use critic_shared::ShowHelp;
use leptos::{ev::keydown, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    path, StaticSegment,
};

use leptos_use::{use_document, use_event_listener};

use critic_components::editor::Editor;
use transcribe::{editor::TranscribeEditor, todo::TranscribeTodoList};

mod admin;
mod transcribe;

const DEFAULT_BUTTON_CLASSES: &str = "text-md m-2 rounded-2xl bg-slate-600 p-2 text-center font-bold text-slate-50 shadow-sm shadow-sky-600 hover:bg-slate-500";

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

#[derive(Clone, PartialEq, Eq)]
enum TopLevelPosition {
    Admin,
    Transcribe,
    Reconcile,
    None,
}

const NAVBAR_BUTTON_CLASSES: &str = "p-2 pl-4 pr-4 hover:bg-slate-500 bg-slate-600 rounded-2xl text-2xl font-bold m-2 text-center shadow-md";
#[component]
fn NavBarButton(
    to: &'static str,
    top_level_pos: ReadSignal<TopLevelPosition>,
    children: Children,
    active_state: &'static TopLevelPosition,
) -> impl IntoView {
    view! {
      <a
        class=NAVBAR_BUTTON_CLASSES
        class=(["text-sky-300", "shadow-slate-300"], move || top_level_pos.read() == *active_state)
        class=(["text-slate-50", "shadow-sky-600"], move || top_level_pos.read() != *active_state)
        href={to}
      >{children()}</a>
    }
}

#[component]
fn NavBar(top_level_pos: ReadSignal<TopLevelPosition>) -> impl IntoView {
    let navbar_help_button_classes = "p-2 pl-4 pr-4 text-slate-50 hover:bg-slate-500 bg-slate-600 rounded-2xl text-2xl font-bold m-2 text-center shadow-md shadow-orange-400/70";

    let help_active = use_context::<RwSignal<ShowHelp>>().expect("App provides show-help context");
    view! {
    <nav class="flex flex-row justify-around bg-black border-b-4 border-slate-600">
      <a href="/logo"><img alt="logo" src="/logo.webp"/></a>
      <NavBarButton to="/transcribe" top_level_pos=top_level_pos active_state=&TopLevelPosition::Transcribe>Transcribe</NavBarButton>
      <NavBarButton to="/reconcile" top_level_pos=top_level_pos active_state=&TopLevelPosition::Reconcile>Reconcile</NavBarButton>
      <NavBarButton to="/admin" top_level_pos=top_level_pos active_state=&TopLevelPosition::Admin>Administer</NavBarButton>
      <span
        on:click=move |_| {
            help_active.update(|a| a.toggle())
        }
        class=navbar_help_button_classes>Help:<span class="ml-2 text-orange-400">ctrl+alt+h</span></span>
    </nav>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let help_active = RwSignal::new(ShowHelp::new(false));
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

    // will be set on page load by the top level routes
    let (top_level_pos, set_top_level_pos) = signal(TopLevelPosition::None);
    provide_context(set_top_level_pos);

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/critic.css"/>

        // sets the document title
        <Title text="critic - textual criticism"/>

        <div class="h-screen w-screen flex flex-col bg-slate-900 text-white">
        // Router
        <Router>
            <NavBar top_level_pos=top_level_pos/>
            <main class="h-0 grow w-full">
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=path!("transcribe") view=TranscribeTodoList/>
                    <Route path=path!("transcribe/:msname/:pagename") view=TranscribeEditor/>
                    <ParentRoute path=path!("admin") view=|| {view!{ <Outlet/> }}>
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
        </div>
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

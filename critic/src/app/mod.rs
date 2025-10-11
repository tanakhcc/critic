use critic_shared::ShowHelp;
use leptos::{ev::keydown, prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    path, StaticSegment,
};

use leptos_use::{use_document, use_event_listener};

use transcribe::{editor::TranscribeEditor, todo::TranscribeTodoList};

mod admin;
pub mod shared;
mod transcribe;

#[derive(Debug)]
struct EmptyError {}
impl core::fmt::Display for EmptyError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "An unspecified error occured.")
    }
}
impl std::error::Error for EmptyError {}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
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
            class=(
                ["text-sky-300", "shadow-slate-300"],
                move || top_level_pos.read() == *active_state,
            )
            class=(
                ["text-slate-50", "shadow-sky-600"],
                move || top_level_pos.read() != *active_state,
            )
            href=to
        >
            {children()}
        </a>
    }
}

#[component]
fn NavBar(top_level_pos: ReadSignal<TopLevelPosition>) -> impl IntoView {
    let navbar_help_button_classes = "p-2 pl-4 pr-4 text-slate-50 hover:bg-slate-500 bg-slate-600 rounded-2xl text-2xl font-bold m-2 text-center shadow-md shadow-orange-400/70";

    let help_active = use_context::<RwSignal<ShowHelp>>().expect("App provides show-help context");
    view! {
        <nav class="flex flex-row justify-around bg-black border-b-4 border-slate-600">
            <a href="/logo">
                <img alt="logo" src="/logo.webp" />
            </a>
            <NavBarButton
                to="/transcribe"
                top_level_pos=top_level_pos
                active_state=&TopLevelPosition::Transcribe
            >
                Transcribe
            </NavBarButton>
            <NavBarButton
                to="/reconcile"
                top_level_pos=top_level_pos
                active_state=&TopLevelPosition::Reconcile
            >
                Reconcile
            </NavBarButton>
            <NavBarButton
                to="/admin"
                top_level_pos=top_level_pos
                active_state=&TopLevelPosition::Admin
            >
                Administer
            </NavBarButton>
            <span
                on:click=move |_| { help_active.update(|a| a.toggle()) }
                class=navbar_help_button_classes
            >
                Help:
                <span class="ml-2 text-orange-400">ctrl+alt+h</span>
            </span>
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
        <Stylesheet id="leptos" href="/pkg/critic.css" />

        // sets the document title
        <Title text="critic - textual criticism" />

        <div class="h-screen w-screen flex flex-col bg-slate-900 text-white">
            // Router
            <Router>
                <NavBar top_level_pos=top_level_pos />
                <main class="h-0 grow w-full">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=StaticSegment("") view=HomePage />
                        <Route path=StaticSegment("view") view=MsViewer />
                        <Route path=path!("transcribe") view=TranscribeTodoList />
                        <Route path=path!("transcribe/:msname/:pagename") view=TranscribeEditor />
                        <ParentRoute
                            path=path!("admin")
                            view=|| {
                                view! { <Outlet /> }
                            }
                        >
                            <Route path=path!("") view=admin::AdminLanding />
                            <admin::AdminRouter />
                        </ParentRoute>
                    </Routes>
                </main>
            </Router>
        </div>
    }
}

#[component]
fn MsViewer() -> impl IntoView {
    let x = RwSignal::new(0);
    let y = RwSignal::new(0);
    let scale = RwSignal::new(1.);
    let in_drag = RwSignal::new(false);
    let saved_real_pixel = RwSignal::new((0, 0));

    // given the position in the enclosing div, return the position in "real live" pixels on the MS
    // i.e. 0, 0 is exactly the top-left point of the MS, no matter how it is currently scaled or
    // translated.
    // The input coordinates need to be in the coordinate system given by the MS images parent -
    // i.e. you may need to add offsets beforehand if your coordinates come from the viewport
    // coordinates of an event.
    let real_pixel = move |x_vp: i32, y_vp: i32| {
        (
            ((x_vp - x.get_untracked()) as f64 / scale.get_untracked()) as i32,
            ((y_vp - y.get_untracked()) as f64 / scale.get_untracked()) as i32,
        )
    };
    // given the real and viewport coordinates, find the offset so that these positions coincide at
    // the current scaling
    let offset_from_real_pixel_at_vp = move |x_r: i32, y_r: i32, x_vp: i32, y_vp: i32| {
        (
            (x_vp as f64 - scale.get_untracked() * x_r as f64) as i32,
            (y_vp as f64 - scale.get_untracked() * y_r as f64) as i32,
        )
    };
    // TODO: also add a middle-mouse-button
    view! {
        <div class="overflow-none flex h-full w-full flex-row">
            <div
                class="w-0 grow overflow-auto border-r-2 border-slate-600"
                style="scrollbar-width: none;"
            >
                <div
                    class="overflow-clip"
                    on:mousedown=move |evt: leptos::ev::MouseEvent| {
                        if evt.buttons() == 4 {
                            in_drag.set(true);
                            saved_real_pixel.set(real_pixel(evt.client_x(), evt.client_y()));
                        }
                    }
                    on:mousemove=move |evt: leptos::ev::MouseEvent| {
                        if in_drag.get_untracked() {
                            if evt.buttons() == 4 {
                                let (x_r, y_r) = saved_real_pixel.get_untracked();
                                let (x_new, y_new) = offset_from_real_pixel_at_vp(
                                    x_r,
                                    y_r,
                                    evt.client_x(),
                                    evt.client_y(),
                                );
                                x.set(x_new);
                                y.set(y_new);
                            } else {
                                in_drag.set(false);
                            }
                        }
                    }
                    on:wheel=move |evt: leptos::ev::WheelEvent| {
                        evt.prevent_default();
                        if evt.ctrl_key() {
                            let effective_scaling_factor = if evt.delta_y() >= 0. {
                                0.8
                            } else {
                                1.25
                            };
                            let x_vp = evt.x();
                            let y_vp = evt.y() + 80;
                            x.update(|curr| {
                                *curr = x_vp
                                    - (effective_scaling_factor * (x_vp - *curr) as f64) as i32
                            });
                            y.update(|curr| {
                                *curr = y_vp
                                    - (effective_scaling_factor * (y_vp - *curr) as f64) as i32
                            });
                            scale.update(|s| *s *= effective_scaling_factor);
                        } else {
                            if evt.shift_key() {
                                x.update(|x| {
                                    *x
                                        += (evt.delta_y() / (scale.get_untracked() as f64).sqrt())
                                            as i32;
                                })
                            } else {
                                y.update(|y| {
                                    *y
                                        += (evt.delta_y() / (scale.get_untracked() as f64).sqrt())
                                            as i32;
                                })
                            }
                        }
                    }
                >
                    <div style:transform=move || format!("translate({}px, {}px)", x.get(), y.get())>
                        <img
                            src="https://ntmss.info/images/webfriendly/HBCE/Hebrew_Manuscripts/Firkovich_Collections/II_B_115/SP%20RNL%20EVR%20II%20B%20115a_Vpage_012.jpg"
                            alt="ms name"
                            style:scale=move || format!("{}", scale.get())
                            style:transform-origin="top left"
                        />
                    </div>
                </div>
            </div>
            <div class="h-full w-1/5 max-w-72 min-w-44 bg-red-200">hi i ams content</div>
        </div>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let show_help = use_context::<RwSignal<ShowHelp>>().expect("Main page provides ShowHelp");

    view! {
        <div class="flex flex-row justify-center">
            <div>
                <h1 class="p-10 text-6xl font-semibold">Welcome to Critic</h1>
                <div class="relative pt-6 text-lg">
                    <p class="text-center">
                        "On many pages, you can press"
                        <span class="ml-2 text-orange-400">"ctrl+alt+h"</span>
                        " to get contextual help. Try it!"
                    </p>
                    <div />
                    <div
                        class="bg-slate-500/50 rounded-lg backdrop-blur-xs absolute inset-0 w-full h-80 text-center"
                        class=(["hidden"], move || !show_help.read().get())
                    >
                        <p class="mt-36">"Just like that!"</p>
                        <p>
                            "To get started, select one of the submenus from the top navigation bar."
                        </p>
                        <p>
                            "Press"<span class="ml-2 text-orange-400">"ctrl+alt+h"</span>
                            " again to close the help overlay."
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}

use critic_shared::{
    urls::{IMAGE_BASE_LOCATION, STATIC_BASE_URL},
    ShowHelp,
};
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

#[server]
async fn get_image_dimensions(
    msname: String,
    pagename: String,
    which: critic_shared::ImageType,
) -> Result<(u32, u32), ServerFnError> {
    use leptos::prelude::use_context;
    let config: std::sync::Arc<critic_server::config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    critic_server::static_files::get_image_dimensions(
        &config.data_directory,
        msname,
        pagename,
        which,
    )
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[component]
fn MsViewer() -> impl IntoView {
    let msname = "IIB115+";
    let pagename = "016";
    let image_base = format!("{STATIC_BASE_URL}{IMAGE_BASE_LOCATION}/{msname}/{pagename}",);
    let image_dimensions = OnceResource::new(get_image_dimensions(
        msname.to_string(),
        pagename.to_string(),
        critic_shared::ImageType::Original,
    ));

    let x = RwSignal::new(0);
    let y = RwSignal::new(0);
    let scale = RwSignal::new(1.);
    let in_drag = RwSignal::new(false);
    let saved_real_pixel = RwSignal::new((0., 0.));
    let view_ref = NodeRef::new();

    // given the position in the enclosing div, return the position in "real live" pixels on the MS
    // i.e. 0, 0 is exactly the top-left point of the MS, no matter how it is currently scaled or
    // translated.
    // The input coordinates need to be in the coordinate system given by the MS images parent -
    // i.e. you may need to add offsets beforehand if your coordinates come from the viewport
    // coordinates of an event.
    let real_pixel = move |x_vp: i32, y_vp: i32| {
        (
            (x_vp - x.get_untracked()) as f64 / scale.get_untracked(),
            (y_vp - y.get_untracked()) as f64 / scale.get_untracked(),
        )
    };
    // given the real and viewport coordinates, find the offset so that these positions coincide at
    // the current scaling
    let offset_from_real_pixel_at_vp = move |x_r: f64, y_r: f64, x_vp: i32, y_vp: i32| {
        (
            (x_vp as f64 - scale.get_untracked() * x_r) as i32,
            (y_vp as f64 - scale.get_untracked() * y_r) as i32,
        )
    };

    // this much of the parent div is always occupied by the image - prevents scrolling/scaling so
    // that the image is not in the viewport anymore
    // Works separately for x and y coordinate.
    let minimal_incidence_factor = 0.2_f64;
    // Sets the offset, but clips the values so that minimal_incidence_factor is respected.
    let set_offset_clipped = move |x_new: i32, y_new: i32| {
        let clipped_offset = if let Some(Ok((dimension_x, dimension_y))) = image_dimensions.get() {
            let div_ref: web_sys::HtmlDivElement = view_ref
                .get_untracked()
                .expect("statically mounted noderef");
            let vp_extent_x = div_ref.offset_width();
            let vp_extent_y = div_ref.offset_height();
            let x_max = ((1. - minimal_incidence_factor) * vp_extent_x as f64) as i32;
            let y_max = ((1. - minimal_incidence_factor) * vp_extent_y as f64) as i32;
            let scale = scale.get_untracked();
            let x_min = ((minimal_incidence_factor - scale as f64) * vp_extent_x as f64) as i32;
            let y_min = (minimal_incidence_factor * vp_extent_y as f64
                - dimension_y as f64 / dimension_x as f64 * vp_extent_x as f64 * scale)
                as i32;
            (x_new.clamp(x_min, x_max), y_new.clamp(y_min, y_max))
        } else {
            (x_new, y_new)
        };
        x.update(|x| *x = clipped_offset.0);
        y.update(|y| *y = clipped_offset.1);
    };

    // this function deals with scrolling and zooming
    let on_wheel = move |evt: leptos::ev::WheelEvent| {
        // do not scroll with browser default, we control scrolling behaviour here
        evt.prevent_default();
        // scaling
        if evt.ctrl_key() {
            let effective_scaling_factor = if evt.delta_y() >= 0. { 0.8 } else { 1.25 };
            let old_scale = scale.get_untracked();
            // get real pixel the mouse points to right now
            let (x_r, y_r) = real_pixel(evt.client_x(), evt.client_y() - 70);
            // update the scale
            scale.update(|s| *s *= effective_scaling_factor);
            let new_scale = old_scale * effective_scaling_factor;
            let x_new = (x_r * (old_scale - new_scale)) as i32 + x.get_untracked();
            let y_new = (y_r * (old_scale - new_scale)) as i32 + y.get_untracked();
            set_offset_clipped(x_new, y_new);
        } else {
            if evt.shift_key() {
                // left-right scrolling
                set_offset_clipped(
                    x.get_untracked()
                        + (evt.delta_y() / (scale.get_untracked() as f64).sqrt()) as i32,
                    y.get_untracked(),
                );
            } else {
                // top-bottom scrolling
                set_offset_clipped(
                    x.get_untracked(),
                    y.get_untracked()
                        + (evt.delta_y() / (scale.get_untracked() as f64).sqrt()) as i32,
                );
            }
        }
    };

    let space_down = RwSignal::new(false);
    // last known mouse position - we need this to get the initial position for the move-on-space-hold
    let last_known_mouse_position = RwSignal::new((0, 0));

    let _down = use_event_listener(view_ref, keydown, move |evt| {
        if evt.key_code() == 32 {
            // on the first space-down, save the starting position for the move and set
            // space_down
            if !space_down.get_untracked() {
                space_down.update(|c| *c = true);
                let (x, y) = last_known_mouse_position.get_untracked();
                saved_real_pixel.set(real_pixel(x, y));
            }
        };
    });
    let _up = use_event_listener(view_ref, leptos::ev::keyup, move |evt| {
        if evt.key_code() == 32 {
            space_down.update(|c| *c = false);
        }
    });

    // this function changes offset while moving
    let on_move = move |evt: leptos::ev::MouseEvent| {
        last_known_mouse_position.set((evt.client_x(), evt.client_y()));
        let space_down_now = space_down.get_untracked();
        if in_drag.get_untracked() || space_down_now {
            if evt.buttons() == 4 || space_down_now {
                let (x_r, y_r) = saved_real_pixel.get_untracked();
                let (x_new, y_new) =
                    offset_from_real_pixel_at_vp(x_r, y_r, evt.client_x(), evt.client_y());
                set_offset_clipped(x_new, y_new);
            // we transformed because of middle-mouse-drag, but middle mouse is no longer pressed -
            // stop dragging
            } else {
                in_drag.set(false);
            }
        }
    };

    // TODO:
    // smaller image for the viewer here?
    view! {
        <div class="overflow-none flex h-full w-full flex-row">
            <div
                class="w-0 grow overflow-auto border-r-2 border-slate-600"
                style="scrollbar-width: none;"
                node_ref=view_ref
                tabindex="0"
                autofocus
            >
                <div
                    class="overflow-clip"
                    on:mousedown=move |evt: leptos::ev::MouseEvent| {
                        if evt.buttons() == 4 {
                            in_drag.set(true);
                            saved_real_pixel.set(real_pixel(evt.client_x(), evt.client_y()));
                        }
                    }
                    on:mousemove=on_move
                    on:wheel=on_wheel
                >
                    <div
                        style:transform=move || {
                            format!(
                                "translate({}px, {}px) scale({})",
                                x.get(),
                                y.get(),
                                scale.get(),
                            )
                        }
                        style:transform-origin="top left"
                    >
                        <img src=format!("{image_base}/original.webp") alt="ms name" />
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

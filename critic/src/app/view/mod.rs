/// Manuscript viewer / editor - shows a manuscript page and allows editing associated data like
/// baselines, transcriptions, reconciliations.
use leptos_router::hooks::use_params;

use critic_shared::urls::{IMAGE_BASE_LOCATION, STATIC_BASE_URL};
use leptos::prelude::*;
use leptos_use::use_event_listener;
use reactive_stores::Store;

use crate::app::shared::{MsParams, PageParams};

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
pub fn MsViewer() -> impl IntoView {
    let ms_param = use_params::<MsParams>();
    let page_param = use_params::<PageParams>();
    let both_names = move || {
        (
            ms_param
                .read_untracked()
                .as_ref()
                .ok()
                .and_then(|x| x.msname.clone()),
            page_param
                .read_untracked()
                .as_ref()
                .ok()
                .and_then(|x| x.pagename.clone()),
        )
    };

    let (Some(msname), Some(pagename)) = both_names() else {
        return leptos::either::Either::Left(
            view! { "Unable to get manuscript and page name from the url." },
        );
    };

    let image_base = format!("{STATIC_BASE_URL}{IMAGE_BASE_LOCATION}/{msname}/{pagename}",);
    let image_dimensions = OnceResource::new(get_image_dimensions(
        msname,
        pagename,
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
            // the actual size of the MS viewer that is on screen in px
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

    let _down = use_event_listener(view_ref, leptos::ev::keydown, move |evt| {
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
    // read from DB (actually add DB for this)
    let regions = Store::new(Regions::default());
    let selected = RwSignal::new(None);
    let overlay_editable = RwSignal::new(false);

    // TODO:
    // smaller image for the viewer here?
    // allow showing boxes overlaid ontop the image, given in real coordinates
    leptos::either::Either::Right(view! {
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
                        <Suspense fallback=|| {}>
                            {move || {
                                image_dimensions
                                    .get()
                                    .map(|dimensions| {
                                        dimensions
                                            .map(|(dim_x, dim_y)| {
                                                view! {
                                                    <MsOverlay
                                                        dim_x=dim_x
                                                        dim_y=dim_y
                                                        regions=regions
                                                        selected=selected
                                                        editable=overlay_editable.read_only()
                                                    />
                                                }
                                            })
                                    })
                            }}
                        </Suspense>
                    </div>
                </div>
            </div>
            <div class="h-full w-1/5 max-w-72 min-w-44 bg-red-200">hi i ams content</div>
        </div>
    })
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Store)]
struct Point {
    x: u32,
    y: u32,
}
impl core::fmt::Display for Point {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{},{}", self.x, self.y)
    }
}

#[derive(Debug, Clone, Store)]
struct Baseline {
    /// uses the same id scheme as kraken, which is why this is a string
    baseline_id: String,
    point1: Point,
    point2: Point,
}

#[derive(Debug, Clone)]
struct Polygon {
    /// closes the polygon between last and first point - i.e. the first point should not be added
    /// as the last point as well
    points: Vec<Point>,
}
impl Polygon {
    /// The points listed in SVG format
    fn point_list(&self) -> String {
        let mut res = String::default();
        for (idx, point) in self.points.iter().enumerate() {
            res.push_str(&point.to_string());
            if idx != self.points.len() - 1 {
                res.push(' ');
            }
        }
        res
    }
}

#[derive(Debug, Clone, Store)]
struct Region {
    /// uses the same id scheme as kraken, which is why this is a string
    region_id: String,
    /// polygon bounding this region
    boundary: Polygon,
    /// Baselines belonging to this Region
    #[store(key: String = |baseline| baseline.baseline_id.clone())]
    baselines: Vec<Baseline>,
    // other useful things: text_type
}

#[derive(Debug, Clone)]
enum DrawableOverlay {
    Baseline(Baseline),
    Region(Region),
}

#[derive(Debug, Clone, Store)]
struct Regions {
    #[store(key: String = |r| r.region_id.clone())]
    regions: Vec<Region>,
}
impl Default for Regions {
    fn default() -> Self {
        Self { regions: vec![] }
    }
}

#[component]
fn MsOverlay(
    /// extent of the coordinate system in x-direction
    dim_x: u32,
    /// extent of the coordinate system in y-direction
    dim_y: u32,
    /// the regions available, including their baselines
    regions: Store<Regions>,
    /// the currently selected region or baseline
    selected: RwSignal<Option<DrawableOverlay>>,
    /// is overlay editing currently allowed or not
    editable: ReadSignal<bool>,
) -> impl IntoView {
    let stroke_width = (dim_x + dim_y) / 150;

    view! {
        <svg
            viewBox=format!("0 0 {dim_x} {dim_y}")
            class="stroke-emerald-400 fill-amber-500 absolute top-0 left-0"
            style:stroke-wdith=format!("{stroke_width}px")
        >
            <For each=move || regions.regions() key=|r| r.clone().region_id().get() let(region)>
                <Region region=region stroke_width=stroke_width />
            </For>
        </svg>
    }
}

#[component]
fn Region(
    region: reactive_stores::AtKeyed<Store<Regions>, Regions, String, Vec<Region>>,
    stroke_width: u32,
) -> impl IntoView {
    view! {
        <polygon points=region.read().boundary.point_list() stroke-width=stroke_width />
        <For
            each=move || region.clone().baselines()
            key=|baseline| baseline.clone().baseline_id().get()
            let(baseline)
        >
            <BaseLine baseline=baseline stroke_width=stroke_width * 2 />
        </For>
    }
}

#[component]
fn BaseLine(
    baseline: reactive_stores::AtKeyed<
        reactive_stores::AtKeyed<Store<Regions>, Regions, String, Vec<Region>>,
        Region,
        String,
        Vec<Baseline>,
    >,
    stroke_width: u32,
) -> impl IntoView {
    view! {
        <line
            x1=baseline.read().point1.x
            y1=baseline.read().point1.y
            x2=baseline.read().point2.x
            y2=baseline.read().point2.y
        />
        <circle cx=baseline.read().point1.x cy=baseline.read().point1.y r=stroke_width * 2 />
        <circle cx=baseline.read().point2.x cy=baseline.read().point2.y r=stroke_width * 2 />
    }
}

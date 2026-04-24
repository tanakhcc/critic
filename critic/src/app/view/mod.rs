/// Manuscript viewer / editor - shows a manuscript page and allows editing associated data like
/// baselines, transcriptions, reconciliations.
use leptos_router::hooks::use_params;

use critic_shared::{
    urls::{IMAGE_BASE_LOCATION, STATIC_BASE_URL},
    Baseline, BaselineStoreFields, Region, RegionStoreFields, SegmentedPage,
    SegmentedPageStoreFields,
};
use leptos::prelude::*;
use leptos_use::use_event_listener;
use reactive_stores::{Store, StoreField};

use crate::app::shared::{MsParams, PageParams};
use crate::app::view::components::SelectedTool;

mod components;

#[server]
async fn get_image_dimensions(
    msname: String,
    pagename: String,
    which: critic_shared::ImageType,
) -> Result<(u32, u32), ServerFnError> {
    use leptos::prelude::use_context;
    let config: std::sync::Arc<critic_config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    critic_server::static_files::get_image_dimensions(
        &config.data_directory,
        msname,
        pagename,
        which,
    )
    .map_err(|e| ServerFnError::new(e.to_string()))
}

#[server]
async fn get_segmentation(
    msname: String,
    pagename: String,
) -> Result<Option<SegmentedPage>, ServerFnError> {
    let config: std::sync::Arc<critic_config::Config> =
        use_context().ok_or(ServerFnError::new("Unable to get config from context"))?;
    let segmentation = critic_db::get_segmentation(&config.db, &msname, &pagename)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if segmentation.regions.iter().any(|r| !r.baselines.is_empty()) {
        Ok(Some(segmentation))
    } else {
        Ok(None)
    }
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
        msname.clone(),
        pagename.clone(),
        critic_shared::ImageType::Original,
    ));
    let segmentation = OnceResource::new(get_segmentation(msname.clone(), pagename));

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

    // this function deals with scrolling
    let on_wheel = move |evt: leptos::ev::WheelEvent| {
        // do not scroll with browser default, we control scrolling behaviour here
        evt.prevent_default();
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

    // the key to the selected element
    let selected = RwSignal::new(None);
    // the active tool
    let tool = RwSignal::new(SelectedTool::Select);

    // TODO:
    // smaller image for the viewer here?
    leptos::either::Either::Right(view! {
        <div class="overflow-none flex h-full w-full flex-row">
            <div
                class="w-0 grow overflow-auto border-r-2 border-slate-600 relative"
                style="scrollbar-width: none;"
                node_ref=view_ref
                tabindex="0"
                autofocus
            >
                <div
                    class="bg-black border-t-2 border-slate-600 absolute bottom-0 w-full z-5"
                    // no scrolling here
                    on:wheel=|evt| {
                        evt.prevent_default();
                    }
                >
                    <components::Information selected=selected.read_only() />
                </div>
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
                        <img src=format!("{image_base}/original.webp") alt=msname />
                        <ErrorBoundary fallback=|errors| {
                            view! {
                                <div>
                                    "Error: failed to get segmentation"
                                    <ul>
                                        {move || {
                                            errors
                                                .get()
                                                .into_iter()
                                                .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                                .collect::<Vec<_>>()
                                        }}
                                    </ul>
                                </div>
                            }
                        }>
                            <Suspense fallback=|| {}>
                                {move || {
                                    image_dimensions
                                        .get()
                                        .map(|dimensions| {
                                            dimensions
                                                .map(|(dim_x, dim_y)| {
                                                    segmentation
                                                        .get()
                                                        .map(|seg_res| {
                                                            seg_res
                                                                .map(|seg| {
                                                                    if let Some(seg) = seg {
                                                                        let regions = Store::new(seg);

                                                                        view! {
                                                                            <MsOverlay
                                                                                dim_x=dim_x
                                                                                dim_y=dim_y
                                                                                regions=regions
                                                                                selected=selected
                                                                                scale=scale.read_only()
                                                                                tool=tool.read_only()
                                                                            />
                                                                        }
                                                                            .into_any()
                                                                    } else {
                                                                        view! { <p>OCR is not yet finished.</p> }.into_any()
                                                                    }
                                                                })
                                                        })
                                                })
                                        })
                                }}
                            </Suspense>
                        </ErrorBoundary>
                    </div>
                </div>
            </div>
            <div class="h-full w-52 bg-black">
                <components::Toolbar on_save=|| {} tool=tool />
                <components::Layers />
            </div>
        </div>
    })
}

/// The entire overlay over the MS image
///
/// The parent div has to scale, but we still need the `scale` because we have to counter-scale
/// some SVG elements.
#[component]
fn MsOverlay(
    /// extent of the unscaled coordinate system in x-direction
    dim_x: u32,
    /// extent of the unscaled coordinate system in y-direction
    dim_y: u32,
    /// the regions available, including their baselines
    regions: Store<SegmentedPage>,
    /// the currently selected region or baseline
    selected: RwSignal<Option<KeyedBaseline>>,
    /// The current scale used in the MS
    scale: ReadSignal<f64>,
    /// the tool currently selected
    tool: ReadSignal<SelectedTool>,
) -> impl IntoView {
    let stroke_width = ((dim_x.pow(2) + dim_y.pow(2)).isqrt() / 250).max(2);

    provide_context(selected);
    provide_context(scale);
    provide_context(tool);
    provide_context(stroke_width);

    view! {
        <svg
            viewBox=format!("0 0 {dim_x} {dim_y}")
            class="stroke-emerald-400 fill-amber-500 absolute top-0 left-0"
            style:stroke-width=move || format!("{}px", (stroke_width as f64 / scale.get()) as u32)
        >
            <For each=move || regions.regions() key=|r| r.clone().id().get() let(region)>
                <Region region=region />
            </For>
        </svg>
    }
}

#[component]
fn Region(
    region: reactive_stores::AtKeyed<Store<SegmentedPage>, SegmentedPage, i64, Vec<Region>>,
) -> impl IntoView {
    view! {
        <polygon points=move || region.read().boundary.point_list() fill="none" stroke="black" />
        <For
            each=move || region.clone().baselines()
            key=|baseline| baseline.clone().id().get()
            let(baseline)
        >
            <BaseLine baseline=baseline />
        </For>
    }
}

type KeyedBaseline = reactive_stores::AtKeyed<
    reactive_stores::AtKeyed<Store<SegmentedPage>, SegmentedPage, i64, Vec<Region>>,
    Region,
    i64,
    Vec<Baseline>,
>;

#[component]
fn BaseLine(baseline: KeyedBaseline) -> impl IntoView {
    let stroke_width = use_context::<u32>().expect("MsOverlay supplies stroke width");
    let tool = use_context::<ReadSignal<SelectedTool>>().expect("MsOverlay supplies selected tool");
    let selected = use_context::<RwSignal<Option<KeyedBaseline>>>()
        .expect("MsOverlay supplies selected element");
    let scale = use_context::<ReadSignal<f64>>().expect("MsOverlay supplies scale");

    view! {
        <g
            class="group"
            on:click=move |_evt| {
                if tool.get() == SelectedTool::Select {
                    selected.set(Some(baseline));
                }
            }
        >
            <line
                x1=move || baseline.read().baseline.0.x
                y1=move || baseline.read().baseline.0.y
                x2=move || baseline.read().baseline.1.x
                y2=move || baseline.read().baseline.1.y
                // stroke-width=format!("{}px", stroke_width * 2)
                class=(["hover:stroke-red-600"], move || tool.get() == SelectedTool::EditLine)
            />
            {move || {
                if tool.get() == SelectedTool::EditLine {
                    Some(
                        view! {
                            <circle
                                cx=move || baseline.read().baseline.0.x
                                cy=move || baseline.read().baseline.0.y
                                r=move || format!("{}px", (stroke_width as f64 / scale.get()))
                                class="fill-orange-400 hover:stroke-red-600"
                            />
                            <circle
                                cx=move || baseline.read().baseline.1.x
                                cy=move || baseline.read().baseline.1.y
                                r=move || format!("{}px", (stroke_width as f64 / scale.get()))
                                class="fill-orange-400 hover:stroke-red-600"
                            />
                        },
                    )
                } else {
                    None
                }
            }}
        </g>
    }
}

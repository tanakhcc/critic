pub mod app;

// these imports are required so that server functions can call server code
#[cfg(feature = "ssr")]
mod server;
#[cfg(feature = "ssr")]
mod shared;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

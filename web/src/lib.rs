mod utils;

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Called by our JS entry point to run the example
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // If the `console_error_panic_hook` feature is enabled this will set a panic hook, otherwise
    // it will do nothing.
    utils::set_panic_hook();
    // console_log::init_with_level(log::Level::Debug).unwrap();
    tracing_wasm::set_as_global_default();

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let renderer = Box::pin(utils::Renderer::attach(
        shared::Model::new("the frontend"),
        body.into(),
    ));
    renderer.as_ref().render();
    Box::leak(Box::new(renderer));

    Ok(())
}

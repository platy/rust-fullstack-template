mod utils;

use std::{intrinsics::transmute, mem::swap, sync::Mutex};

use wasm_bindgen::prelude::*;

use lignin_dom::{diff::update_child_nodes, lignin::{bumpalo::Bump, Node}, load::load_element};

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

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let mut bump_a = Box::leak(Box::new(Bump::new()));
    let mut bump_b = Box::leak(Box::new(Bump::new()));
    let mut previous: Option<Node> = None;
    let container = body;
    let render = Mutex::new(move || {
        let vdom_old = previous.take().unwrap_or_else(|| {
            load_element(&container, &bump_a).content[0]
        });
        let vdom: Node = shared::view(&bump_b, shared::Model("the frontend"));
        web_sys::console::log_1(&JsValue::from_str(&format!("vdom is {:?}, should be {:?}", &vdom_old, &vdom)));
        unsafe {
            //SAFETY:
            // This formally detaches the lifetime of `vdom` from `bump_b`, so that the latter can be swapped with `bump_a`.
            // Since the reset of `bump_a` only happens after `previous` is overwritten (and after the previous DOM has been updated away),
            // there shouldn't be any dangling references.
            update_child_nodes(&[vdom_old], &[vdom], &container); // <--
            previous = Some(transmute::<Node<'_>, Node<'_>>(vdom));
            bump_a.reset();
        }
        swap(&mut bump_a, &mut bump_b);
    });
    let render: &dyn Fn() = Box::leak(Box::new(move || {
        // Inefficient if called directly, should schedule a render instead.
        let render = &mut *render.lock().unwrap();
        render()
    }));
    render(); 

    Ok(())
}

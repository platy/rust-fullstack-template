mod utils;

use std::{intrinsics::transmute, mem::swap, sync::Mutex};

use wasm_bindgen::prelude::*;
use lignin_dom::{diff::DomDiffer, load::{Allocator, load_element}};
use lignin::{Node};
use bumpalo::Bump;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct BumpAllocator<'a>(&'a Bump);

impl<'a> Allocator<'a> for BumpAllocator<'a> {
    fn allocate<T>(&self, instance: T) -> &'a T {
        self.0.alloc(instance)
    }

    fn allocate_slice<T>(&self, iter: &mut dyn ExactSizeIterator<Item = T>) -> &'a [T] {
        self.0.alloc_slice_fill_iter(iter)
    }
}

struct Renderer {
    /// Spare bump space, always empty at rest
    bump_spare: Bump,
    bump: Bump,
    /// Previously rendered vdom, allocated in `bump`
    previous: Vec<Node<'static, lignin::ThreadBound>>,
    differ: DomDiffer,
    model: shared::Model,
}

impl Renderer {
    pub fn attach(model: shared::Model, container: web_sys::Element) -> Renderer {
        let bump = Bump::new();
        let initial: Node<lignin::ThreadBound> = load_element(&BumpAllocator(&bump), &container).content.into();
        let previous = unsafe {
            //SAFETY:
            // This formally detaches the lifetime of `previous` from `bump`, so that both can be owned by the same struct.
            // `previous` must not be allowed to outlive `bump`, or live beyond it's next reset.
            transmute::<Node<'_, _>, Node<'_, _>>(initial)
        };
        let mut differ = DomDiffer::new_for_element_child_nodes(container.into());
        Renderer {
            bump_spare: Bump::new(),
            bump,
            previous: vec![previous],
            differ,
            model,
        }
    }

    pub fn render(&mut self) {
        let vdom: Node<_> = shared::view(&self.bump_spare, &self.model);
        // web_sys::console::log_1(&JsValue::from_str(&format!("vdom is {:?}, should be {:?}", &vdom_old[0], &vdom)));
        unsafe {
            //SAFETY:
            // This formally detaches the lifetime of `vdom` from `bump_b`, so that the latter can be swapped with `bump_a`.
            // Since the reset of `bump_a` only happens after `previous` is overwritten (and after the previous DOM has been updated away),
            // there shouldn't be any dangling references.
            self.differ.update_child_nodes(&self.previous, &[vdom], 20);
            self.previous[0] = transmute::<Node<'_, _>, Node<'_, _>>(vdom);
            // `bump` can be reset as the old `previous` which was allocated within it is now dropped
            self.bump.reset();
        }
        swap(&mut self.bump, &mut self.bump_spare);
    }
}

// Called by our JS entry point to run the example
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // If the `console_error_panic_hook` feature is enabled this will set a panic hook, otherwise
    // it will do nothing.
    utils::set_panic_hook();
    // console_log::init_with_level(log::Level::Debug);

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    let mut renderer = Renderer::attach(shared::Model("the frontend"), body.into());
    renderer.render();

    Ok(())
}

use std::{cell::{Cell, RefCell}, fmt, intrinsics::transmute, mem::swap, pin::Pin, ptr::NonNull, rc::Rc};

use bumpalo::Bump;
use js_sys::Function;
use lignin::Node;
use lignin_dom::{
    diff::DomDiffer,
    load::{load_element, Allocator},
};
use shared::{Model, ModelData};
use wasm_bindgen::{prelude::*, JsCast};

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

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
    previous: [Node<'static, lignin::ThreadBound>; 1],
    differ: DomDiffer,
}

impl Renderer {
    /// Renders the model into a vdom and updates the real dom to match
    pub fn render(&mut self, model: &Model) {
        let vdom = shared::view(
            &self.bump_spare,
            model,
        );
        // web_sys::console::log_1(&JsValue::from_str(&format!("vdom is {:?}, should be {:?}", &vdom_old[0], &vdom)));
        self.differ.update_child_nodes(&self.previous, &[vdom], 20);
        unsafe {
            //SAFETY:
            // This formally detaches the lifetime of `vdom` from `bump_spare`, so that the latter can be swapped with `bump`.
            // Since the reset of `bump` only happens after `previous` is overwritten (and after the previous DOM has been updated away),
            // there shouldn't be any dangling references.
            self.previous[0] = transmute::<Node<'_, _>, Node<'_, _>>(vdom);
            // `bump` can be reset as the old `previous` which was allocated within it is now dropped
            self.bump.reset();
        }
        swap(&mut self.bump, &mut self.bump_spare);
    }

    /// Attach the renderer to a DOM element, lignin reads the element, and generates a VDOM for it
    pub fn attach(container: web_sys::Element) -> Self {
        let bump = Bump::new();
        let initial: Node<lignin::ThreadBound> = load_element(&BumpAllocator(&bump), &container)
            .content
            .into();
        let previous = unsafe {
            //SAFETY:
            // This formally detaches the lifetime of `previous` from `bump`, so that both can be owned by the same struct.
            // `previous` must not be allowed to outlive `bump`, or live beyond it's next reset.
            transmute::<Node<'_, _>, Node<'_, _>>(initial)
        };
        Renderer {
            bump_spare: Bump::new(),
            bump,
            previous: [previous],
            differ: DomDiffer::new_for_element_child_nodes(container),
        }
    }
}

pub struct RenderLoop {
    render_state: RefCell<Renderer>,
    render_scheduled: Cell<bool>,
    render_callback: RefCell<Option<Function>>,

    model: Pin<Box<shared::Model>>,
}

impl fmt::Debug for RenderLoop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RenderLoop")
    }
}

impl RenderLoop {
    // #[tracing::instrument]
    pub fn attach(model: ModelData, container: web_sys::Element) -> Pin<Rc<RenderLoop>> {
        let model = Model::new(model);
        let r = Rc::pin(RenderLoop {
            render_scheduled: Cell::new(false),
            render_callback: RefCell::new(None),
            render_state: RefCell::new(Renderer::attach(container)),
            model,
        });
        {
            let r2 = r.clone();
            r.model.set_render_scheduler(Box::new(move || r2.schedule_render()));
        }
        r
    }

    #[tracing::instrument]
    pub fn render(&self) {
        self.render_scheduled.set(false);
        self.render_state.borrow_mut().render(&self.model);
    }

    #[tracing::instrument]
    pub fn schedule_render(&self) {
        let renderer = NonNull::from(self);
        let mut render_callback = self.render_callback.borrow_mut();
        let render_callback = render_callback.get_or_insert_with(|| {
            Closure::wrap(
                Box::new(move || unsafe { renderer.as_ref().render() }) as Box<dyn FnMut()>
            )
            .into_js_value()
            .unchecked_into()
        });
        if !self.render_scheduled.get() {
            let window = web_sys::window().expect("no global `window` exists");
            window.request_animation_frame(&render_callback).unwrap();
        }
    }
}

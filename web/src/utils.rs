use std::{
    cell::{Cell, RefCell},
    intrinsics::transmute,
    mem::swap,
    pin::Pin,
    ptr::NonNull,
};

use bumpalo::Bump;
use js_sys::Function;
use lignin::{
    web::Event, CallbackRegistration, EventBinding, EventBindingOptions, Node, ThreadBound,
};
use lignin_dom::{
    diff::DomDiffer,
    load::{load_element, Allocator},
};
use shared::{Helper, Model};
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

struct RenderState {
    /// Spare bump space, always empty at rest
    bump_spare: Bump,
    bump: Bump,
    /// Previously rendered vdom, allocated in `bump`
    previous: Vec<Node<'static, lignin::ThreadBound>>,
    differ: DomDiffer,
}

impl RenderState {
    pub fn render(&mut self, model: &Model, renderer: &Renderer) {
        let vdom = shared::view(
            &self.bump_spare,
            model,
            &ActualHelper {
                renderer,
                bump: &self.bump_spare,
            },
        );
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

pub struct Renderer {
    render_state: RefCell<RenderState>,
    render_scheduled: Cell<bool>,
    render_callback: RefCell<Option<Function>>,

    model: shared::Model,
    /// Keeps the lignin event registrations, on drop, lignin will cancel the callback
    registrations: RefCell<Vec<CallbackRegistration<CallbackReceiver, fn(lignin::web::Event)>>>,
}

impl Renderer {
    pub fn attach(model: Model, container: web_sys::Element) -> Pin<Box<Renderer>> {
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
        let r = Renderer {
            render_scheduled: Cell::new(false),
            render_callback: RefCell::new(None),
            render_state: RefCell::new(RenderState {
                bump_spare: Bump::new(),
                bump,
                previous: vec![previous],
                differ: DomDiffer::new_for_element_child_nodes(container),
            }),
            model,
            registrations: RefCell::new(Vec::with_capacity(20)),
        };
        Box::pin(r)
    }

    pub fn render(&self) {
        self.render_scheduled.set(false);
        self.render_state.borrow_mut().render(&self.model, self)
    }

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

struct CallbackReceiver {
    /// Access to the parent Renderer, for event handling
    renderer: NonNull<Renderer>,
    /// Function for handling an event
    callback: fn(&mut Model, Event),
}

struct ActualHelper<'r, 'b> {
    renderer: &'r Renderer,
    bump: &'b Bump,
}

impl<'r, 'a> Helper<'a> for ActualHelper<'r, 'a> {
    fn event_binding(
        &self,
        name: &'a str,
        options: EventBindingOptions,
        callback: fn(&mut Model, Event),
    ) -> &'a [EventBinding<'a, ThreadBound>] {
        let renderer = self.renderer;
        let receiver = self.bump.alloc(CallbackReceiver {
            renderer: NonNull::from(&*renderer), // the receiver needs to be pinned and I guess it needs to be the Pin context of the Renderer (though not sure) in that case i think i have to use map_unchecked. I can't then use bump to store the receiver.  I'm tooo tired
            callback,
        });
        let callback_reg = CallbackRegistration::<_, fn(lignin::web::Event)>::new(
            Pin::new(receiver),
            |cr, event| unsafe {
                let CallbackReceiver {
                    mut renderer,
                    callback,
                    ..
                } = *cr;
                let renderer = renderer.as_mut();
                let model = &mut renderer.model;
                callback(model, event);
                renderer.schedule_render();
            },
        );
        let b = EventBinding {
            name,
            options,
            callback: callback_reg.to_ref_thread_bound(),
        };
        renderer.registrations.borrow_mut().push(callback_reg);
        self.bump.alloc_slice_copy(&[b])
    }
}

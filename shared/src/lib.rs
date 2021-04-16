use lignin::{Element, web::Event, Node, ThreadBound, ElementCreationOptions, EventBinding, EventBindingOptions, CallbackRegistration};
use bumpalo::Bump;
use std::{cell::Cell, pin::Pin};

pub struct Model {
    area: &'static str,
    counter: Cell<u32>,
    reg: Cell<Option<lignin::CallbackRegistration<Model, fn(lignin::web::Event)>>>,
}

impl Model {
    pub fn new(area: &'static str) -> Self {
        Model {
            area,
            counter: Cell::new(0),
            reg: Cell::new(None),
        }
    }
}

fn increment(receiver: *const Model, _event: Event) {
    unsafe {
        log::info!("increment {:?}", receiver);

        let receiver = &*receiver;
        receiver.counter.set(receiver.counter.get() + 1);
    }
}

pub fn view<'a>(bump: &'a Bump, model: Pin<&Model>) -> Node<'a, ThreadBound> {
    log::info!("Rendering view");
    let callback_reg = CallbackRegistration::<_, fn(lignin::web::Event)>::new(model, increment);
    let v = bump.alloc(Element {
        name: "P",
        attributes: &[],
        content: Node::Text {
            text: bumpalo::format!(in bump, "Hello from {} of your full-stack Rust app! Counter is {}", model.area, model.counter.get()).into_bump_str(),
            dom_binding: None,
        },
        event_bindings: bump.alloc_slice_copy(&[
            EventBinding {
                name: "click",
                options: EventBindingOptions::new(),
                callback: callback_reg.to_ref_thread_bound(),
            }
        ]),
        creation_options: ElementCreationOptions::new(),
    }).as_html();
    model.reg.set(Some(callback_reg));
    v
}

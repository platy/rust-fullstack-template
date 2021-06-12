use std::{cell::{RefCell}, pin::Pin, sync::atomic::AtomicU32};

use lignin::{CallbackRegistration, Element, ElementCreationOptions, EventBinding, EventBindingOptions, Node, ThreadBound, web::Event};
use bumpalo::Bump;

// #[derive(Debug)]
pub struct ModelData {
    area: &'static str,
    counter: AtomicU32,
}

impl ModelData {
    pub fn new(area: &'static str) -> ModelData {
        ModelData {
            area,
            counter: AtomicU32::new(0),
        }
    }
}

// #[derive(Debug)]
pub struct Model<S> {
    data: ModelData,
    on_click_registration: RefCell<Option<CallbackRegistration<Self, fn(lignin::web::Event)>>>,
    schedule_render: RefCell<Option<S>>,
}

impl<S: Fn()> Model<S> {
    pub fn new(data: ModelData) -> Pin<Box<Self>> {
        let m = Box::pin(Model {
            data,
            on_click_registration: RefCell::new(None),
            schedule_render: RefCell::new(None),
        });
        // Register a callback at that receiver
        m.on_click_registration.try_borrow_mut().unwrap().replace(CallbackRegistration::<_, fn(lignin::web::Event)>::new(
            m.as_ref(),
            Self::on_click,
        ));
        m
    }

    fn on_click(receiver: *const Self, _event: Event) {
        log::info!("clicked");

        let receiver = unsafe {
            &*receiver
        };
        let counter = &receiver.data.counter;

        let v = counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        if let Some(schedule_render) = receiver.schedule_render.try_borrow().unwrap().as_ref() {
            (schedule_render)();
        }

        log::info!("incremented {}", v);
    }

    pub fn set_render_scheduler(&self, f: S) {
        *self.schedule_render.borrow_mut() = Some(f);
    }
}

pub fn view<'a, S: Fn()>(bump: &'a Bump, model: &Model<S>) -> Node<'a, ThreadBound> {
    log::info!("Rendering view");
    bump.alloc(Element {
        name: "P",
        attributes: &[],
        content: Node::Text {
            text: bumpalo::format!(in bump, "Hello from {} of your full-stack Rust app! Counter is {}", model.data.area, model.data.counter.load(std::sync::atomic::Ordering::Relaxed)).into_bump_str(),
            dom_binding: None,
        },
        event_bindings: bump.alloc([EventBinding {
            name: "click",
            options: EventBindingOptions::new(),
            callback: model.on_click_registration.try_borrow().unwrap().as_ref().unwrap().to_ref_thread_bound(),
        }]),
        creation_options: ElementCreationOptions::new(),
    }).as_html()
}

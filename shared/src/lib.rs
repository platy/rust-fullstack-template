use lignin::{Element, ElementCreationOptions, EventBinding, EventBindingOptions, Node, ThreadBound, web::Event};
use bumpalo::Bump;

pub struct Model {
    area: &'static str,
    counter: u32,
}

impl Model {
    pub fn new(area: &'static str) -> Self {
        Model {
            area,
            counter: 0,
        }
    }

    fn on_click(&mut self, _event: Event) {
        log::info!("increment {}", self.counter);

        self.counter += 1;
    }
}

pub fn view<'a>(bump: &'a Bump, model: &Model, helper: &dyn Helper<'a>) -> Node<'a, ThreadBound> {
    log::info!("Rendering view");
    bump.alloc(Element {
        name: "P",
        attributes: &[],
        content: Node::Text {
            text: bumpalo::format!(in bump, "Hello from {} of your full-stack Rust app! Counter is {}", model.area, model.counter).into_bump_str(),
            dom_binding: None,
        },
        event_bindings: helper.event_binding("click", EventBindingOptions::new(), Model::on_click),
        creation_options: ElementCreationOptions::new(),
    }).as_html()
}

pub trait Helper<'a> {
    fn event_binding(&self, name: &'a str, options: EventBindingOptions, callback: fn(&mut Model, Event)) -> &'a[EventBinding<'a, ThreadBound>];
}

impl<'a> Helper<'a> for () {
    fn event_binding(&self, _name: &'a str, _options: EventBindingOptions, _callback: fn(&mut Model, Event)) -> &'a[EventBinding<'a, ThreadBound>] {
        &[]
    }
}
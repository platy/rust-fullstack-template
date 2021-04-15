use lignin::{Element, Node, ThreadBound, ElementCreationOptions};
use bumpalo::Bump;

pub struct Model(pub &'static str);

pub fn view<'a>(bump: &'a Bump, model: &Model) -> Node<'a, ThreadBound> {
    bump.alloc(Element {
        name: "P",
        attributes: &[],
        content: Node::Text {
            text: bumpalo::format!(in bump, "Hello from {} of your full-stack Rust app!", model.0).into_bump_str(),
            dom_binding: None,
        },
        event_bindings: &[],
        creation_options: ElementCreationOptions::new(),
    }).as_html()
}

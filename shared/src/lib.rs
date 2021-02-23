use lignin::{bumpalo::Bump, Node};

pub struct Model(pub &'static str);

pub fn view<'a>(bump: &'a Bump, model: Model) -> Node<'a> {
    use lignin_schema::*;

    Node::Element(
        bump.alloc(bump.alloc_with(|| {
            p(
                &[],
                bump.alloc_slice_copy(&[Node::Text(
                    bumpalo::format!(in bump, "Hello from {} of your full-stack Rust app!", model.0).into_bump_str()
                )]),
                &[],
            )
        }))
    )
}

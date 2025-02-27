use crate::component::{ComponentData, State};

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a element.",
    label = "Expected valid element"
)]
pub trait Element<C: ComponentData> {
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node;
    #[inline(always)]
    fn render(self, ctx: &State<C>) -> web_sys::Node
    where
        Self: Sized,
    {
        Box::new(self).render_box(ctx)
    }
}

impl<C: ComponentData> Element<C> for web_sys::Node {
    fn render_box(self: Box<Self>, _ctx: &State<C>) -> web_sys::Node {
        *self
    }
}

pub struct Comment;

impl<C: ComponentData> Element<C> for Comment {
    fn render_box(self: Box<Self>, _ctx: &State<C>) -> web_sys::Node {
        web_sys::Comment::new()
            .expect("Failed to make comment")
            .into()
    }
}

#[cfg(feature = "element_unit")]
impl<C: ComponentData> Element<C> for () {
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        Element::<C>::render(Comment, ctx)
    }
}

impl<T: Element<C>, C: ComponentData> Element<C> for Option<T> {
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        match *self {
            Some(element) => element.render(ctx),
            None => Element::<C>::render(Comment, ctx),
        }
    }
}

impl<T: Element<C>, E: Element<C>, C: ComponentData> Element<C> for Result<T, E> {
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        match *self {
            Ok(element) => element.render(ctx),
            Err(element) => element.render(ctx),
        }
    }
}

impl<C: ComponentData> Element<C> for &str {
    fn render_box(self: Box<Self>, _ctx: &State<C>) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(*self));
        text.into()
    }
}

impl<C: ComponentData> Element<C> for String {
    fn render_box(self: Box<Self>, _ctx: &State<C>) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(&self));
        text.into()
    }
}

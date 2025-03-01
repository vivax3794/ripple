use crate::signal::RenderingState;
use crate::state::{ComponentData, State};

#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a element.",
    label = "Expected valid element"
)]
pub trait Element<C>: 'static {
    #[doc(hidden)]
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node;

    #[doc(hidden)]
    fn render(self, ctx: &mut State<C>, render_state: &mut RenderingState) -> web_sys::Node
    where
        Self: Sized,
    {
        Box::new(self).render_box(ctx, render_state)
    }
}

impl<C> Element<C> for web_sys::Node {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        *self
    }
}

pub struct Comment;

impl<C> Element<C> for Comment {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        web_sys::Comment::new()
            .expect("Failed to make comment")
            .into()
    }
}

#[cfg(feature = "element_unit")]
impl<C: ComponentData> Element<C> for () {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        Element::<C>::render(Comment, ctx, render_state)
    }
}

impl<T: Element<C>, C: ComponentData> Element<C> for Option<T> {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        match *self {
            Some(element) => element.render(ctx, render_state),
            None => Element::<C>::render(Comment, ctx, render_state),
        }
    }
}

impl<T: Element<C>, E: Element<C>, C: ComponentData> Element<C> for Result<T, E> {
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        match *self {
            Ok(element) => element.render(ctx, render_state),
            Err(element) => element.render(ctx, render_state),
        }
    }
}

impl<C> Element<C> for &'static str {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(*self));
        text.into()
    }
}

impl<C> Element<C> for String {
    fn render_box(
        self: Box<Self>,
        _ctx: &mut State<C>,
        _render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(&self));
        text.into()
    }
}

macro_rules! int_element {
    ($T:ident) => {
        impl<C> Element<C> for $T {
            fn render_box(
                self: Box<Self>,
                _ctx: &mut State<C>,
                _render_state: &mut RenderingState,
            ) -> web_sys::Node {
                let mut buffer = itoa::Buffer::new();
                let result = buffer.format(*self);

                let text = web_sys::Text::new().expect("Failed to make text");
                text.set_text_content(Some(result));
                text.into()
            }
        }
    };
}

macro_rules! int_elements {
    ($($T:ident),*) => {
        $(int_element!{$T})*
    };
}

int_elements! {u8, u16, u32, u64, u128, i8, i16, i32, i128, usize, isize }

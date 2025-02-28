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
    #[inline]
    fn render_box(self: Box<Self>, _ctx: &State<C>) -> web_sys::Node {
        *self
    }
}

pub struct Comment;

impl<C: ComponentData> Element<C> for Comment {
    #[inline]
    fn render_box(self: Box<Self>, _ctx: &State<C>) -> web_sys::Node {
        web_sys::Comment::new()
            .expect("Failed to make comment")
            .into()
    }
}

#[cfg(feature = "element_unit")]
impl<C: ComponentData> Element<C> for () {
    #[inline]
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        Element::<C>::render(Comment, ctx)
    }
}

impl<T: Element<C>, C: ComponentData> Element<C> for Option<T> {
    #[inline]
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        match *self {
            Some(element) => element.render(ctx),
            None => Element::<C>::render(Comment, ctx),
        }
    }
}

impl<T: Element<C>, E: Element<C>, C: ComponentData> Element<C> for Result<T, E> {
    #[inline]
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        match *self {
            Ok(element) => element.render(ctx),
            Err(element) => element.render(ctx),
        }
    }
}

impl<C: ComponentData> Element<C> for &str {
    #[inline]
    fn render_box(self: Box<Self>, _ctx: &State<C>) -> web_sys::Node {
        let text = web_sys::Text::new().expect("Failed to make text");
        text.set_text_content(Some(*self));
        text.into()
    }
}

impl<C: ComponentData> Element<C> for String {
    #[inline]
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        let x: &str = &self;
        Element::<C>::render(x, ctx)
    }
}

macro_rules! int_element {
    ($T:ident) => {
        impl<C: ComponentData> Element<C> for $T {
            #[inline]
            fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
                let mut buffer = itoa::Buffer::new();
                let result = buffer.format(*self);
                Element::<C>::render(result, ctx)
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

pub trait Element<C> {
    fn render(self, ctx: &mut C) -> web_sys::Node;
}

pub struct Comment;

impl<C> Element<C> for Comment {
    fn render(self, _ctx: &mut C) -> web_sys::Node {
        web_sys::Comment::new().unwrap().into()
    }
}

impl<T: Element<C>, C> Element<C> for Option<T> {
    fn render(self, ctx: &mut C) -> web_sys::Node {
        match self {
            Some(element) => element.render(ctx),
            None => Comment.render(&mut ()),
        }
    }
}

impl<C> Element<C> for &str {
    fn render(self, _ctx: &mut C) -> web_sys::Node {
        let text = web_sys::Text::new().unwrap();
        text.set_text_content(Some(self));
        text.into()
    }
}

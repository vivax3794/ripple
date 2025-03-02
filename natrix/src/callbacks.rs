use crate::element::Element;
use crate::render_callbacks::ReactiveNode;
use crate::signal::RenderingState;
use crate::state::{ComponentData, State};

impl<F, C, R> Element<C> for F
where
    F: Fn(&State<C>) -> R + 'static,
    R: Element<C> + 'static,
    C: ComponentData,
{
    fn render_box(
        self: Box<Self>,
        ctx: &mut State<C>,
        render_state: &mut RenderingState,
    ) -> web_sys::Node {
        let (hook, node) = ReactiveNode::create_inital(Box::new(self), ctx);

        render_state.keep_alive.push(Box::new(hook.0));
        node
    }
}

pub trait Event<C> {
    fn func(self) -> Box<dyn Fn(&mut State<C>)>;
}
impl<C, F: Fn(&mut State<C>) + 'static> Event<C> for F {
    fn func(self) -> Box<dyn Fn(&mut State<C>)> {
        Box::new(self)
    }
}

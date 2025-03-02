use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;

use crate::element::Element;
use crate::get_document;
use crate::signal::{RcDep, RcDepWeak, ReactiveHook, RenderingState};
use crate::state::{ComponentData, KeepAlive, State};
use crate::utils::{RcCmpPtr, WeakCmpPtr};

type NodeCallback<C, E> = Box<dyn Fn(&State<C>) -> E>;

pub(crate) struct ReactiveNode<C: ComponentData, E> {
    callback: NodeCallback<C, E>,
    target_node: web_sys::Node,
    keep_alive: Vec<KeepAlive>,
}

impl<C: ComponentData, E: Element<C>> ReactiveNode<C, E> {
    fn render_inplace(&mut self, ctx: &mut State<C>, you: RcDepWeak<C>) {
        let new_node = self.render(ctx, you);

        let parent = self.target_node.parent_node().expect("No parent found");
        parent
            .replace_child(&new_node, &self.target_node)
            .expect("Failed to replace node");
        self.target_node = new_node;
    }

    fn render(
        &mut self,
        ctx: &mut State<C>,
        you: crate::utils::WeakCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>,
    ) -> web_sys::Node {
        ctx.clear();
        let element = (self.callback)(ctx);
        ctx.reg_dep(you);

        self.keep_alive.clear();
        let mut state = RenderingState {
            keep_alive: &mut self.keep_alive,
        };
        element.render(ctx, &mut state)
    }
}

impl<C: ComponentData, E: Element<C>> ReactiveHook<C> for ReactiveNode<C, E> {
    fn update(&mut self, ctx: &mut State<C>, you: RcDepWeak<C>) {
        self.render_inplace(ctx, you);
    }
}

struct DummyHook;
impl<C: ComponentData> ReactiveHook<C> for DummyHook {
    fn update(&mut self, _ctx: &mut State<C>, _you: RcDepWeak<C>) {}
}

impl<C: ComponentData, E: Element<C>> ReactiveNode<C, E> {
    // This is the best I could figure out
    pub(crate) fn create_inital(
        callback: NodeCallback<C, E>,
        ctx: &mut State<C>,
    ) -> (RcDep<C>, web_sys::Node) {
        let dummy_node = get_document()
            .body()
            .expect("WHAT?")
            .dyn_into()
            .expect("HUH?!");

        let result_owned: RcDep<C> = RcCmpPtr(Rc::new(RefCell::new(Box::new(DummyHook))));
        let result_weak = Rc::downgrade(&result_owned.0);

        let mut this = Self {
            callback,
            target_node: dummy_node,
            keep_alive: Vec::new(),
        };

        let node = this.render(ctx, WeakCmpPtr(result_weak));
        this.target_node = node.clone();

        *result_owned.0.borrow_mut() = Box::new(this);

        (result_owned, node)
    }
}

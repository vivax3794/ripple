use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::element::Element;

struct DataTracker<T, C: ComponentData> {
    data: T,
    written: bool,
    read: Cell<bool>,
    depdencies: Vec<RenderCallback<C>>,
}

impl<T, C: ComponentData> DataTracker<T, C> {
    fn new(data: T) -> Self {
        Self {
            data,
            written: false,
            read: Cell::new(false),
            depdencies: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.written = false;
        self.read.set(false);
    }

    fn register_callback(&mut self, callback: RenderCallback<C>) {
        if self.read.get() {
            self.depdencies.push(callback);
        }
    }

    fn update(&mut self, data: &mut C) {
        if self.written {
            for callback in &mut self.depdencies {
                callback.update(data);
            }
        }
    }
}

impl<T, C: ComponentData> Deref for DataTracker<T, C> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.read.set(true);
        &self.data
    }
}
impl<T, C: ComponentData> DerefMut for DataTracker<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written = true;
        &mut self.data
    }
}

pub struct RenderCallback<C: ComponentData> {
    element: Rc<dyn Fn(&mut C) -> web_sys::Node>,
    target_node: web_sys::Node,
}

impl<C: ComponentData> Clone for RenderCallback<C> {
    fn clone(&self) -> Self {
        Self {
            element: Rc::clone(&self.element),
            target_node: self.target_node.clone(),
        }
    }
}

impl<C: ComponentData> RenderCallback<C> {
    fn update(&mut self, data: &mut C) {
        let new_node = (self.element)(data);
        let parent = self.target_node.parent_node().unwrap();
        parent.replace_child(&self.target_node, &new_node).unwrap();
        self.target_node = new_node;
    }
}

pub trait ComponentData: Sized {
    fn clear_state(&mut self);
    fn register_dependency(&mut self, callback: RenderCallback<Self>);
    fn update(&mut self);
    fn weak(&self) -> Weak<Self>;
}

pub trait ComponentBase {
    type Data: ComponentData;
}

pub trait Component: ComponentBase + Sized {
    fn render() -> impl Element<Self::Data>;
}

impl<F, C, R> Element<C> for F
where
    F: Fn(&C) -> R + 'static,
    R: Element<C>,
    C: ComponentData,
{
    fn render(self, ctx: &mut C) -> web_sys::Node {
        ctx.clear_state();
        let element = self(ctx);

        let node = element.render(ctx);
        let callback = RenderCallback {
            element: Rc::new(move |data| self(data).render(data)),
            target_node: node.clone(),
        };
        ctx.register_dependency(callback);
        node
    }
}

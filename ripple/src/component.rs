use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::element::Element;
use crate::utils::RcCmpPtr;

pub(crate) trait SmallAny {}
impl<T> SmallAny for T {}

pub struct DataTracker<T, C: ComponentData> {
    data: T,
    written: Cell<bool>,
    read: Cell<bool>,
    callbacks: RefCell<HashSet<RcCmpPtr<RenderCallback<C>>>>,
    new_callbacks: RefCell<Vec<RcCmpPtr<RenderCallback<C>>>>,
    remove_callbacks: RefCell<Vec<RcCmpPtr<RenderCallback<C>>>>,
}

impl<T, C: ComponentData> DataTracker<T, C> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            written: Cell::new(false),
            read: Cell::new(false),
            callbacks: RefCell::default(),
            new_callbacks: RefCell::default(),
            remove_callbacks: RefCell::default(),
        }
    }
}

pub trait DataTrackerMethods<C: ComponentData> {
    fn clear(&self);
    fn register_callback(&self, callback: RcCmpPtr<RenderCallback<C>>);
    fn drain_queue(&self);
    fn remove(&self, callback: RcCmpPtr<RenderCallback<C>>);
    fn update(&self, data: &State<C>);
}

impl<C: ComponentData, T> DataTrackerMethods<C> for DataTracker<T, C> {
    #[inline(always)]
    fn clear(&self) {
        self.written
            .set(false);
        self.read
            .set(false);
    }

    #[inline]
    fn register_callback(&self, callback: RcCmpPtr<RenderCallback<C>>) {
        if self
            .read
            .get()
        {
            self.new_callbacks
                .borrow_mut()
                .push(callback);
        }
    }

    #[inline]
    fn remove(&self, callback: RcCmpPtr<RenderCallback<C>>) {
        self.remove_callbacks
            .borrow_mut()
            .push(callback);
    }

    fn drain_queue(&self) {
        let mut callbacks = self
            .callbacks
            .borrow_mut();
        let mut new = self
            .new_callbacks
            .borrow_mut();
        let mut remove = self
            .remove_callbacks
            .borrow_mut();

        for callback in new.drain(..) {
            callbacks.insert(callback);
        }
        for callback in remove.drain(..) {
            callbacks.remove(&callback);
        }
    }

    #[inline]
    fn update(&self, data: &State<C>) {
        if self
            .written
            .get()
        {
            for callback in self
                .callbacks
                .borrow()
                .iter()
            {
                RenderCallback::update(callback, data);
            }
        }
    }
}

impl<T, C: ComponentData> Deref for DataTracker<T, C> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.read
            .set(true);
        &self.data
    }
}
impl<T, C: ComponentData> DerefMut for DataTracker<T, C> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written
            .set(true);
        &mut self.data
    }
}

pub(crate) struct RenderCallback<C: ComponentData> {
    element: Box<dyn Fn(&State<C>) -> Box<dyn Element<C>>>,
    target_node: RefCell<web_sys::Node>,
    children_callbacks: RefCell<Vec<RcCmpPtr<Self>>>,
    children_events: RefCell<Vec<Box<dyn SmallAny>>>,
}

impl<C: ComponentData> RenderCallback<C> {
    pub(crate) fn update(this: &RcCmpPtr<Self>, ctx: &State<C>) {
        for callback in this
            .children_callbacks
            .borrow_mut()
            .drain(..)
        {
            ctx.remove(callback);
        }

        ctx.clear_state();
        let element = (this.element)(ctx);
        ctx.register_dependency(this.clone());

        ctx.new_stack();
        let new_node = element.render_box(ctx);
        let (new_callback, new_event) = ctx.pop_stack();

        *this
            .children_callbacks
            .borrow_mut() = new_callback;
        *this
            .children_events
            .borrow_mut() = new_event;

        let mut target_node = this
            .target_node
            .borrow_mut();
        let parent = target_node
            .parent_node()
            .expect("No parent found");
        parent
            .replace_child(&new_node, &target_node)
            .expect("Failed to replace node");
        *target_node = new_node;
    }
}

pub trait ComponentData: Sized + 'static {
    #[doc(hidden)]
    fn references(&self) -> Vec<&dyn DataTrackerMethods<Self>>;
}

pub struct State<T: ComponentData> {
    pub(crate) data: T,
    this: Option<Weak<RefCell<Self>>>,
    callback_stack: RefCell<Vec<Vec<RcCmpPtr<RenderCallback<T>>>>>,
    event_stack: RefCell<Vec<Vec<Box<dyn SmallAny>>>>,
}

impl<T: ComponentData> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: ComponentData> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub type S<C> = State<<C as ComponentBase>::Data>;

impl<T: ComponentData> State<T> {
    pub(crate) fn new(data: T) -> Rc<RefCell<Self>> {
        let this = Self {
            data,
            this: None,
            callback_stack: RefCell::default(),
            event_stack: RefCell::default(),
        };
        let this = Rc::new(RefCell::new(this));

        this.borrow_mut()
            .this = Some(Rc::downgrade(&this));

        this
    }

    #[inline(always)]
    pub(crate) fn weak(&self) -> Weak<RefCell<Self>> {
        self.this
            .as_ref()
            .expect("Weak not set")
            .clone()
    }

    pub(crate) fn clear_state(&self) {
        for x in self
            .data
            .references()
        {
            x.clear();
        }
    }

    pub(crate) fn register_dependency(&self, callback: RcCmpPtr<RenderCallback<T>>) {
        for x in self
            .data
            .references()
        {
            x.register_callback(callback.clone());
        }

        for stack in self
            .callback_stack
            .borrow_mut()
            .iter_mut()
        {
            stack.push(callback.clone());
        }
    }

    pub(crate) fn remove(&self, callback: RcCmpPtr<RenderCallback<T>>) {
        for x in self
            .data
            .references()
        {
            x.remove(callback.clone());
        }
    }

    #[inline]
    fn new_stack(&self) {
        self.callback_stack
            .borrow_mut()
            .push(Vec::new());
        self.event_stack
            .borrow_mut()
            .push(Vec::new());
    }

    #[inline]
    fn pop_stack(&self) -> (Vec<RcCmpPtr<RenderCallback<T>>>, Vec<Box<dyn SmallAny>>) {
        let callback = self
            .callback_stack
            .borrow_mut()
            .pop()
            .expect("Callback stack stack empty");
        let event = self
            .event_stack
            .borrow_mut()
            .pop()
            .expect("Stack empty");
        (callback, event)
    }

    #[inline]
    pub(crate) fn register_event(&self, event: Box<dyn SmallAny>) {
        self.event_stack
            .borrow_mut()
            .last_mut()
            .expect("No stack")
            .push(event);
    }

    fn drain_queue(&self) {
        for x in self
            .data
            .references()
        {
            x.drain_queue();
        }
    }

    pub(crate) fn update(&self) {
        for x in self
            .data
            .references()
        {
            x.update(self);
        }
        self.drain_queue();
    }
}

#[diagnostic::on_unimplemented(
    message = "Type `{Self}` is not a component.",
    note = "add `#[derive(Component)]` to the struct"
)]
pub trait ComponentBase: Sized {
    type Data: ComponentData;
    fn into_data(self) -> Self::Data;

    #[inline(always)]
    fn into_state(self) -> Rc<RefCell<State<Self::Data>>> {
        State::new(self.into_data())
    }
}

pub trait Component: ComponentBase {
    fn render() -> impl Element<Self::Data>;
}

impl<F, C, R> Element<C> for F
where
    F: Fn(&State<C>) -> R + 'static,
    R: Element<C> + 'static,
    C: ComponentData,
{
    fn render_box(self: Box<Self>, ctx: &State<C>) -> web_sys::Node {
        let node: web_sys::Node = web_sys::Comment::new()
            .expect("Failed to create temp comment")
            .into();

        let callback = RenderCallback {
            element: Box::new(move |ctx| Box::new(self(ctx))),
            target_node: RefCell::new(node.clone()),
            children_callbacks: RefCell::default(),
            children_events: RefCell::default(),
        };
        let callback = RcCmpPtr(Rc::new(callback));

        let parent = web_sys::window()
            .expect("Failed to get window")
            .document()
            .expect("Failed to get document")
            .create_element("div")
            .expect("Faield to create temp parent");

        parent
            .append_child(&node)
            .expect("Failed to append child");
        RenderCallback::update(&callback, ctx);

        parent
            .first_child()
            .expect("Child was just inserted")
    }
}

/// Mounts the component at the target id
/// Replacing the element with the component
/// This should be the entry point to your application
///
/// **WARNING:** This method implicitly leaks the memory of the root component
#[inline(always)]
pub fn mount_component<C: Component>(component: C, target_id: &'static str) {
    let data = component.into_state();
    let element = C::render();

    let borrow_data = data.borrow();
    borrow_data.new_stack();
    let node = element.render(&data.borrow());
    let (_, root_event_handlers) = borrow_data.pop_stack();
    borrow_data.drain_queue();

    let document = web_sys::window()
        .expect("Failed to get window")
        .document()
        .expect("Failed to get document");
    let target = document
        .get_element_by_id(target_id)
        .expect("Failed to get mount point");
    target
        .replace_with_with_node_1(&node)
        .expect("Failed to replace mount point");

    drop(borrow_data);

    // This is the entry point, this component should be alive FOREVER
    std::mem::forget(data);
    std::mem::forget(root_event_handlers);
}

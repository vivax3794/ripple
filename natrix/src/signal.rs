use std::cell::{Cell, RefCell};
use std::ops::{Deref, DerefMut};

use crate::state::{ComponentData, KeepAlive, State};
use crate::utils::{HashSet, RcCmpPtr, WeakCmpPtr};

pub type RcDep<C> = RcCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>;
pub type RcDepWeak<C> = WeakCmpPtr<RefCell<Box<dyn ReactiveHook<C>>>>;

pub(crate) struct RenderingState<'s> {
    pub(crate) keep_alive: &'s mut Vec<KeepAlive>,
}

pub struct Signal<T, C> {
    data: T,
    written: bool,
    read: Cell<bool>,
    deps: HashSet<RcDepWeak<C>>,
}

pub trait SignalMethods<C> {
    fn clear(&mut self);
    fn register_dep(&mut self, dep: RcDepWeak<C>);
    fn deps(&mut self) -> &mut HashSet<RcDepWeak<C>>;
    fn changed(&self) -> bool;
}

impl<T, C> Signal<T, C> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            written: false,
            read: Cell::new(false),
            deps: HashSet::default(),
        }
    }
}

impl<T, C> SignalMethods<C> for Signal<T, C> {
    fn clear(&mut self) {
        self.written = false;
        self.read.set(false);
    }

    fn register_dep(&mut self, dep: RcDepWeak<C>) {
        if self.read.get() {
            self.deps.insert(dep);
        }
    }

    fn changed(&self) -> bool {
        self.written
    }

    fn deps(&mut self) -> &mut HashSet<RcDepWeak<C>> {
        &mut self.deps
    }
}

impl<T, C> Deref for Signal<T, C> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.read.set(true);
        &self.data
    }
}
impl<T, C> DerefMut for Signal<T, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.written = true;
        &mut self.data
    }
}

pub(crate) trait ReactiveHook<C: ComponentData> {
    fn update(&mut self, ctx: &mut State<C>, you: RcDepWeak<C>);
}

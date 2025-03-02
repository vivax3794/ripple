use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::component::ComponentBase;
use crate::signal::{RcDepWeak, SignalMethods};
use crate::utils::{HashMap, RcCmpPtr, SmallAny};

#[doc(hidden)]
pub trait ComponentData: Sized + 'static {
    #[doc(hidden)]
    fn signals(&self) -> Vec<&dyn SignalMethods<Self>>;
    #[doc(hidden)]
    fn signals_mut(&mut self) -> Vec<&mut dyn SignalMethods<Self>>;
}

pub(crate) type KeepAlive = Box<dyn SmallAny>;

pub struct State<T> {
    pub(crate) data: T,
    this: Option<Weak<RefCell<Self>>>,
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

pub type S<C> = State<<C as ComponentBase>::Data>;

impl<T> State<T> {
    pub(crate) fn new(data: T) -> Rc<RefCell<Self>> {
        let this = Self { data, this: None };
        let this = Rc::new(RefCell::new(this));

        this.borrow_mut().this = Some(Rc::downgrade(&this));

        this
    }

    pub(crate) fn weak(&self) -> Weak<RefCell<Self>> {
        self.this.as_ref().expect("Weak not set").clone()
    }
}

impl<T: ComponentData> State<T> {
    pub(crate) fn clear(&mut self) {
        for signal in self.data.signals_mut() {
            signal.clear();
        }
    }

    pub(crate) fn reg_dep(&mut self, dep: RcDepWeak<T>) {
        for signal in self.data.signals_mut() {
            signal.register_dep(dep.clone());
        }
    }

    pub(crate) fn update(&mut self) {
        #[allow(clippy::mutable_key_type)]
        let mut hooks = HashMap::default();
        for signal in self.data.signals_mut() {
            if signal.changed() {
                signal.deps().retain(|hook| {
                    if let Some(rc_hook) = hook.0.upgrade() {
                        hooks.insert(RcCmpPtr(rc_hook), hook.clone());
                        true
                    } else {
                        false
                    }
                });
            }
        }

        for (hook, hook_weak) in hooks {
            hook.0.borrow_mut().update(self, hook_weak);
        }
    }
}

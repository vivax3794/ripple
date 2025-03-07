//! Types for handling the component state

use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use crate::component::ComponentBase;
use crate::signal::{RcDepWeak, SignalMethods};
use crate::utils::{HashSet, SmallAny};

/// Trait implemented on the reactive struct generated by the derive macro
pub trait ComponentData: Sized + 'static {
    /// The type of the returned signal fields.
    /// This should be a [...; N]
    type FieldRef<'a>: IntoIterator<Item = &'a mut dyn SignalMethods<Self>>;

    /// Returns mutable references to the signals
    #[doc(hidden)]
    fn signals_mut(&mut self) -> Self::FieldRef<'_>;
}

/// Alias for `Box<dyn SmallAny>`
/// for keeping specific objects alive in memory such as `Closure` and `Rc`
pub(crate) type KeepAlive = Box<dyn SmallAny>;

/// The core component state, stores all framework data
pub struct State<T> {
    /// The user (macro) defined reactive struct
    pub(crate) data: T,
    /// A weak reference to ourself, so that event handlers can easially get a weak reference
    /// without having to pass it around in every api
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

/// A type alias for `State<C::Data>`, should be prefered in closure argument hints.
/// such as `|ctx: &S<Self>| ...`
pub type S<C> = State<<C as ComponentBase>::Data>;

/// A type alias for `RenderCtx<C::Data>`, should be prefered in closure argument hints.
/// such as `|ctx: R<Self>| ...`
pub type R<'c, C> = RenderCtx<'c, <C as ComponentBase>::Data>;

impl<T> State<T> {
    /// Create a new instance of the state, returning a `Rc` to it
    pub(crate) fn new(data: T) -> Rc<RefCell<Self>> {
        let this = Self { data, this: None };
        let this = Rc::new(RefCell::new(this));

        this.borrow_mut().this = Some(Rc::downgrade(&this));

        this
    }

    /// Get a weak reference to this state
    #[inline]
    pub(crate) fn weak(&self) -> Weak<RefCell<Self>> {
        self.this.as_ref().expect("Weak not set").clone()
    }
}

impl<T: ComponentData> State<T> {
    /// Clear all signals
    pub(crate) fn clear(&mut self) {
        for signal in self.data.signals_mut() {
            signal.clear();
        }
    }

    /// Register a dependency for all read signals
    pub(crate) fn reg_dep(&mut self, dep: &RcDepWeak<T>) {
        for signal in self.data.signals_mut() {
            signal.register_dep(dep.clone());
        }
    }

    /// Loop over signals and update any depdant hooks for changed signals
    pub(crate) fn update(&mut self) {
        #[allow(clippy::mutable_key_type)]
        let mut hooks = HashSet::default();
        for signal in self.data.signals_mut() {
            if signal.changed() {
                for hook in signal.deps().drain(..) {
                    hooks.insert(hook);
                }
            }
        }

        let mut current_valid_hooks = Vec::new();
        for hook in hooks {
            if let Some(hook_strong) = hook.0.upgrade() {
                hook_strong.borrow_mut().drop_children_early();
                current_valid_hooks.push(hook);
            }
        }
        for hook in current_valid_hooks {
            if let Some(hook_strong) = hook.0.upgrade() {
                hook_strong.borrow_mut().update(self, &hook);
            }
        }
    }
}

/// Wrapper around a mutable state that only allows read-only access
///
/// This holds a mutable state to faciliate a few rendering features such as `.watch`
pub struct RenderCtx<'c, C>(pub(crate) &'c mut State<C>);

impl<C> Deref for RenderCtx<'_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}

/// Async features
#[cfg(feature = "async")]
pub mod async_impl {
    use std::cell::{RefCell, RefMut};
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::rc::{Rc, Weak};

    use ouroboros::self_referencing;

    use super::{ComponentData, State};

    /// A combiend `Weak` and `RefCell` that facilities upgrading and borrowing as a shared
    /// operation
    pub struct AsyncCtx<T> {
        /// The `Weak<RefCell<T>>` in question
        inner: Weak<RefCell<State<T>>>,
    }

    // We put a bound on `'p` so that users are not able to store the upgraded reference (unless
    // they want to use ouroboros themself to store it alongside the weak).
    #[self_referencing]
    struct AsyncRefInner<'p, T: 'static> {
        rc: Rc<RefCell<State<T>>>,
        lifetime: PhantomData<&'p ()>,
        #[borrows(rc)]
        #[covariant]
        reference: RefMut<'this, State<T>>,
    }

    /// a `RefMut` that also holds a `Rc`.
    /// See the `WeakRefCell::borrow_mut` on drop semantics and safety
    #[cfg_attr(feature = "nightly", must_not_suspend)]
    pub struct AsyncRef<'p, T: ComponentData + 'static>(AsyncRefInner<'p, T>);

    impl<T: ComponentData> AsyncCtx<T> {
        /// Borrow this `Weak<RefCell<...>>`, this will create a `Rc` for as long as the borrow is
        /// active. Returns `None` if the component was dropped. Its recommended to use the
        /// following construct to safely cancel async tasks:
        /// ```rust
        /// let Some(borrow) = ctx.borrow_mut() else {return;};
        /// // ...
        /// drop(borrow);
        /// foo().await;
        /// let Some(borrow) = ctx.borrow_mut() else {return;};
        /// // ...
        /// ```
        ///
        /// # Reactivity
        /// Calling this function clears the internal reactive flags (which is safe as long as the
        /// borrow safety rules below are followed).
        /// Once this value is dropped it will trigger a reactive update for any changed fields.
        ///
        /// # Panics
        /// This function will panic if a borrow already exsists.
        ///
        /// # Borrow Safety
        /// The framework guarantees that it will never hold a borrow between event calls.
        /// This means the only source of panics is if you are holding a borrow when you yield to
        /// the event loop, i.e you should *NOT* hold this value across `.await` points.
        /// framework will regulary borrow the state on any registerd event handler trigger, for
        /// example a user clicking a button.
        ///
        /// Keeping this type across an `.await` point or otherwise leading control to the event
        /// loop while the borrow is active could also lead to reactivity failrues and desyncs, and
        /// should be considerd UB (not ub as in compile ub, but as in this framework makes no
        /// guarantees about what state the reactivity system will be in)
        ///
        /// ## Nightly
        /// The nightly feature flag enables a lint to detect this misuse.
        /// See the [Features]() chapther for details on how to set it up (it requires a bit more
        /// setup than just turning on the feature flag).
        pub fn borrow_mut(&mut self) -> Option<AsyncRef<'_, T>> {
            let rc = self.inner.upgrade()?;
            let mut borrow = AsyncRefInner::new(rc, PhantomData, |rc| rc.borrow_mut());
            borrow.with_reference_mut(|ctx| ctx.clear());
            Some(AsyncRef(borrow))
        }
    }

    impl<T: ComponentData> Deref for AsyncRef<'_, T> {
        type Target = State<T>;

        fn deref(&self) -> &Self::Target {
            self.0.borrow_reference()
        }
    }
    impl<T: ComponentData> DerefMut for AsyncRef<'_, T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.with_reference_mut(|cell| &mut **cell)
        }
    }

    impl<T: ComponentData> Drop for AsyncRef<'_, T> {
        fn drop(&mut self) {
            self.0.with_reference_mut(|ctx| {
                ctx.update();
            });
        }
    }

    impl<T: ComponentData> State<T> {
        /// Get a wrapper around `Weak<RefCell<T>>` which provides a safer api that aligns with
        /// framework assumptions.
        fn get_async_ctx(&mut self) -> AsyncCtx<T> {
            AsyncCtx { inner: self.weak() }
        }

        /// Spawn a async task in the local event loop, which will run on the next possible moment.
        // This is `&mut` to make sure it cant be called in render callbacks.
        pub fn use_async<C, F>(&mut self, func: C)
        where
            C: FnOnce(AsyncCtx<T>) -> F,
            F: Future<Output = ()> + 'static,
        {
            wasm_bindgen_futures::spawn_local(func(self.get_async_ctx()));
        }
    }
}

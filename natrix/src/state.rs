//! Types for handling the component state

use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use slotmap::{SlotMap, new_key_type};

use crate::component::ComponentBase;
use crate::render_callbacks::DummyHook;
use crate::signal::{PreUpdateResult, ReactiveHook, RenderingState, SignalMethods};
use crate::utils::SmallAny;

/// The cap for how many loops `update` will do before panicing
///
/// Used to avoid reference cycles
const UPDATE_ITERATION_CAP: u8 = 2;

/// Trait implemented on the reactive struct generated by the derive macro
pub trait ComponentData: Sized + 'static {
    /// The type of the returned signal fields.
    /// This should be a [...; N]
    type FieldRef<'a>: IntoIterator<Item = &'a mut dyn SignalMethods>;
    /// The type used to represent a snapshot of the signal state.
    type SignalState;

    /// Returns mutable references to the signals
    #[doc(hidden)]
    fn signals_mut(&mut self) -> Self::FieldRef<'_>;
    /// Clear signals and return the current state
    fn pop_signals(&mut self) -> Self::SignalState;
    /// Set signals to the given state
    fn set_signals(&mut self, state: Self::SignalState);
}

/// Alias for `Box<dyn SmallAny>`
/// for keeping specific objects alive in memory such as `Closure` and `Rc`
pub(crate) type KeepAlive = Box<dyn SmallAny>;

new_key_type! { pub(crate) struct HookKey; }

/// The core component state, stores all framework data
pub struct State<T> {
    /// The user (macro) defined reactive struct
    pub(crate) data: T,
    /// A weak reference to ourself, so that event handlers can easially get a weak reference
    /// without having to pass it around in every api
    this: Option<Weak<RefCell<Self>>>,
    /// Reactive hooks
    pub(crate) hooks: SlotMap<HookKey, Box<dyn ReactiveHook<T>>>,
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
pub type R<'c, 's, C> = RenderCtx<'c, 's, <C as ComponentBase>::Data>;

impl<T> State<T> {
    /// Create a new instance of the state, returning a `Rc` to it
    pub(crate) fn new(data: T) -> Rc<RefCell<Self>> {
        let this = Self {
            data,
            this: None,
            hooks: SlotMap::default(),
        };
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
    pub(crate) fn reg_dep(&mut self, dep: HookKey) {
        for signal in self.data.signals_mut() {
            signal.register_dep(dep);
        }
    }

    /// Remove the hook from the slotmap, runs the function on it, then puts it back.
    ///
    /// This is to allow mut access to both the hook and self
    fn run_with_hook_and_self<F>(&mut self, hook: HookKey, func: F) -> Option<()>
    where
        F: FnOnce(&mut Self, &mut Box<dyn ReactiveHook<T>>),
    {
        let slot_ref = self.hooks.get_mut(hook)?;
        let mut temp_hook: Box<dyn ReactiveHook<T>> = Box::new(DummyHook);
        std::mem::swap(slot_ref, &mut temp_hook);

        func(self, &mut temp_hook);

        let slot_ref = self.hooks.get_mut(hook)?;
        *slot_ref = temp_hook;

        Some(())
    }

    /// Loop over signals and update any depdant hooks for changed signals
    pub(crate) fn update(&mut self) {
        #[allow(clippy::mutable_key_type)]
        let mut hooks = HashSet::new();
        for signal in self.data.signals_mut() {
            if signal.changed() {
                for hook in signal.deps() {
                    hooks.insert(hook);
                }
            }
        }

        let mut hooks_two = HashSet::default();
        let mut iteration = 0;
        loop {
            let mut new_hook_added = false;
            for hook_key in hooks.drain() {
                self.run_with_hook_and_self(hook_key, |ctx, hook| {
                    match hook.pre_update(ctx, hook_key) {
                        PreUpdateResult::KeepMe => {
                            if let Some(invalid_hooks) = hook.drop_deps() {
                                for invalid_hook in invalid_hooks {
                                    let _ = ctx.hooks.remove(invalid_hook);
                                }
                            }
                            hooks_two.insert(hook_key);
                        }
                        PreUpdateResult::RemoveMe => {}
                        PreUpdateResult::RegisterThenRemove(dep) => {
                            hooks_two.insert(dep);
                            new_hook_added = true;
                        }
                    }
                });
            }

            std::mem::swap(&mut hooks, &mut hooks_two);
            if !new_hook_added {
                break;
            }

            iteration += 1;
            assert!(
                (iteration <= UPDATE_ITERATION_CAP),
                "Update spent {UPDATE_ITERATION_CAP} iterations on resolving pre updates. You might have a reference cycle"
            );
        }

        for hook_key in hooks {
            self.run_with_hook_and_self(hook_key, |ctx, hook| {
                hook.update(ctx, hook_key);
            });
        }
    }

    /// Get the unwraped data referenced by this guard
    pub fn get<F, A>(&self, guard: &Guard<F>) -> A
    where
        F: Fn(&Self) -> A,
    {
        (guard.getter)(self)
    }
}

/// Wrapper around a mutable state that only allows read-only access
///
/// This holds a mutable state to faciliate a few rendering features such as `.watch`
pub struct RenderCtx<'c, 's, C> {
    /// The inner context
    pub(crate) ctx: &'c mut State<C>,
    /// The render state for this state
    pub(crate) render_state: RenderingState<'s>,
}

impl<C> Deref for RenderCtx<'_, '_, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.ctx.data
    }
}

impl<C: ComponentData> RenderCtx<'_, '_, C> {
    /// Calculate the value using the function and cache it using `clone`.
    /// Then whenever any signals read in the function are modified re-run the function and check
    /// if the new result is different.
    /// Only reruns the caller when the item is different.
    pub fn watch<T, F>(&mut self, func: F) -> T
    where
        F: Fn(&State<C>) -> T + 'static,
        T: PartialEq + Clone + 'static,
    {
        let signal_state = self.ctx.pop_signals();

        let result = func(self.ctx);

        let hook = WatchState {
            calc_value: Box::new(func),
            last_value: result.clone(),
            dep: self.render_state.parent_dep,
        };
        let me = self.ctx.hooks.insert(Box::new(hook));
        self.ctx.reg_dep(me);
        self.render_state.hooks.push(me);

        self.ctx.set_signals(signal_state);

        result
    }

    /// Get the unwraped data referenced by this guard
    pub fn get<F, A>(&self, guard: &Guard<F>) -> A
    where
        F: Fn(&State<C>) -> A,
    {
        self.ctx.get(guard)
    }
}

/// The wather hook / signal
struct WatchState<F, T> {
    /// Function to calculate the state
    calc_value: F,
    /// The previous cached value
    last_value: T,
    /// The depdency that owns us.
    dep: HookKey,
}

impl<C, F, T> ReactiveHook<C> for WatchState<F, T>
where
    C: ComponentData,
    T: PartialEq + Clone,
    F: Fn(&State<C>) -> T,
{
    fn pre_update(&mut self, ctx: &mut State<C>, you: HookKey) -> PreUpdateResult {
        ctx.clear();
        let new_value = (self.calc_value)(ctx);
        ctx.reg_dep(you);

        if new_value == self.last_value {
            PreUpdateResult::RemoveMe
        } else {
            PreUpdateResult::RegisterThenRemove(self.dep)
        }
    }
}

/// This guard ensures that when it is in scope the data it was created for is `Some`
#[derive(Clone, Copy)]
pub struct Guard<F> {
    /// The closure for getting the value from a ctx
    getter: F,
}

/// Get a guard handle that can be used to retrive the `Some` variant of a option without having to
/// use `.unwrap`.
/// Should be used to achive find-grained reactivity (internally this uses `.watch` on `.is_some()`)
///
/// # Why?
/// The usecase can be seen by considering this logic:
/// ```rust
/// if let Some(value) = *ctx.value {
///     e::div().text(value)
/// } else {
///     e::div().text("Is none")
/// }
/// ```
/// The issue here is that the outer div (which might be a more expensive structure to create) is
/// recreated everytime `value` changes, even if it is `Some(0) -> Some(1)`
/// This is where you might reach for `ctx.watch`, and in fact that works perfectly:
/// ```rust
/// if ctx.watch(|ctx| ctx.value.is_some()) {
///     e::div().text(|ctx: R<Self>| ctx.value.unwrap())
/// } else {
///     e::div().text("Is none")
/// }
/// ```
/// And this works, Now a change from `Some(0)` to `Some(1)` will only run the inner closure and
/// the outer div is reused. but there is one downside, we need `.unwrap` because the inner closure is
/// technically isolated, and this is ugly, and its easy to do by accident. and you might forget
/// the outer condition.
///
/// This is where guards come into play:
/// ```rust
/// if let Some(value_guard) = guard_option!(ctx.value) {
///     e::div().text(move |ctx: R<Self>| ctx.get(&value_guard))
/// } else {
///     e::div().text("Is none")
/// }
/// ```
/// Here `value_guard` is actually not the value at all, its a lightweight value thats can be
/// captured my child closures and basically is a way to say "I know that in this context this
/// value is `Some`"
///
/// Internally this uses `ctx.watch` and `.unwrap` (which should never fail)
#[macro_export]
macro_rules! guard_option {
    ($ctx:ident. $($getter:tt)+) => {
        if $ctx.watch(move |ctx| ctx.$($getter)+.is_some()) {
            Some(::natrix::macro_ref::Guard::new(
                move |ctx: &::natrix::macro_ref::S<Self>| ctx.$($getter)+.unwrap(),
            ))
        } else {
            None
        }
    };
}

/// Get a guard handle that can be used to retrive the `Ok` variant of a option without having to
/// use `.unwrap`, or the `Err` variant.
#[macro_export]
macro_rules! guard_result {
    ($ctx:ident. $($getter:tt)+) => {
        if $ctx.watch(move |ctx| ctx.$($getter)+.is_ok()) {
            Ok(::natrix::macro_ref::Guard::new(
                move |ctx: &::natrix::macro_ref::S<Self>| ctx.$($getter)+.unwrap(),
            ))
        } else {
            Err(::natrix::macro_ref::Guard::new(
                move |ctx: &::natrix::macro_ref::S<Self>| ctx.$($getter)+.unwrap_err(),
            ))
        }
    };
}

impl<F> Guard<F> {
    #[doc(hidden)]
    pub fn new(getter: F) -> Self {
        Self { getter }
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

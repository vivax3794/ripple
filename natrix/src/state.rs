//! Types for handling the component state

use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender};
use ouroboros::self_referencing;
use slotmap::{SlotMap, new_key_type};

use crate::component::Component;
use crate::render_callbacks::DummyHook;
use crate::signal::{ReactiveHook, RenderingState, SignalMethods, UpdateResult};
use crate::utils::{self, SmallAny, debug_expect};

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
pub struct State<T: Component> {
    /// The user (macro) defined reactive struct
    pub(crate) data: T::Data,
    /// A weak reference to ourself, so that event handlers can easially get a weak reference
    /// without having to pass it around in every api
    this: Option<Weak<RefCell<Self>>>,
    /// Reactive hooks
    hooks: SlotMap<HookKey, (Box<dyn ReactiveHook<T>>, u64)>,
    /// The next value to use in the insertion order map
    next_insertion_order_value: u64,
    /// The sender for the parent listning to this
    send_to_parent: Option<UnboundedSender<T::EmitMessage>>,
}

impl<T: Component> Deref for State<T> {
    type Target = T::Data;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T: Component> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// A type alias for `State<C::Data>`, should be preferred in closure argument hints.
/// such as `|ctx: S<Self>| ...`
pub type E<'c, C> = &'c mut State<C>;

/// A type alias for `RenderCtx<C::Data>`, should be preferred in closure argument hints.
/// such as `|ctx: R<Self>| ...`
pub type R<'a, 'c, C> = &'a mut RenderCtx<'c, C>;

impl<T: Component> State<T> {
    /// Create a new instance of the state, returning a `Rc` to it
    pub(crate) fn new(data: T::Data) -> Rc<RefCell<Self>> {
        let this = Self {
            data,
            this: None,
            hooks: SlotMap::default(),
            next_insertion_order_value: 0,
            send_to_parent: None,
        };
        let this = Rc::new(RefCell::new(this));

        this.borrow_mut().this = Some(Rc::downgrade(&this));

        this
    }

    /// Get a weak reference to this state
    #[expect(clippy::expect_used, reason = "This is always set in the `new` method")]
    fn weak(&self) -> Weak<RefCell<Self>> {
        self.this.as_ref().expect("Weak not set").clone()
    }

    /// Clear all signals
    pub(crate) fn clear(&mut self) {
        for signal in self.data.signals_mut() {
            signal.clear();
        }
    }

    /// Insert a hook and keep track of insertion order
    pub(crate) fn insert_hook(&mut self, hook: Box<dyn ReactiveHook<T>>) -> HookKey {
        let key = self.hooks.insert((hook, self.next_insertion_order_value));
        self.next_insertion_order_value = debug_expect!(
            self.next_insertion_order_value.checked_add(1),
            or(0),
            "Overflowed hook insertion value"
        );
        key
    }

    /// Update the value for a hook
    pub(crate) fn set_hook(&mut self, key: HookKey, hook: Box<dyn ReactiveHook<T>>) {
        if let Some(slot) = self.hooks.get_mut(key) {
            slot.0 = hook;
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
    fn run_with_hook_and_self<F, R>(&mut self, hook: HookKey, func: F) -> Option<R>
    where
        F: FnOnce(&mut Self, &mut Box<dyn ReactiveHook<T>>) -> R,
    {
        let slot_ref = self.hooks.get_mut(hook)?;
        let mut temp_hook: Box<dyn ReactiveHook<T>> = Box::new(DummyHook);
        std::mem::swap(&mut slot_ref.0, &mut temp_hook);

        let res = func(self, &mut temp_hook);

        let slot_ref = self.hooks.get_mut(hook)?;
        slot_ref.0 = temp_hook;

        Some(res)
    }

    /// Loop over signals and update any depdant hooks for changed signals
    pub(crate) fn update(&mut self) {
        let mut hooks = Vec::new();
        for signal in self.data.signals_mut() {
            if signal.changed() {
                hooks.extend(signal.deps());
            }
        }

        hooks.sort_by_key(|hook_key| Some(self.hooks.get(*hook_key)?.1));
        hooks.dedup_by_key(|hook_key| Some(self.hooks.get(*hook_key)?.1));
        hooks.reverse();

        while let Some(hook_key) = hooks.pop() {
            self.run_with_hook_and_self(hook_key, |ctx, hook| match hook.update(ctx, hook_key) {
                UpdateResult::Nothing => {}
                UpdateResult::RunHook(dep) => {
                    hooks.push(dep);
                }
                UpdateResult::DropHooks(deps) => {
                    for dep in deps {
                        drop_hook(ctx, dep);
                    }
                }
            });
        }
    }

    /// Get the unwrapped data referenced by this guard
    pub fn get<'s, F, R>(&'s self, guard: &Guard<F>) -> &'s R
    where
        F: Fn(&'s Self) -> &'s R,
    {
        (guard.getter)(self)
    }

    /// Get the unwrapped data referenced by this guard, but owned
    pub fn get_owned<F, R>(&self, guard: &Guard<F>) -> R
    where
        F: Fn(&Self) -> R,
    {
        (guard.getter)(self)
    }

    /// Emit a message to the parent component
    pub fn emit(&mut self, msg: T::EmitMessage) {
        if let Some(sender) = self.send_to_parent.as_ref() {
            debug_expect!(
                sender.unbounded_send(msg),
                "Failed to send message on channel, parent likely dropped"
            );
        }
    }

    /// Register a new sender from the parent component
    pub(crate) fn register_parent(&mut self, sender: UnboundedSender<T::EmitMessage>) {
        self.send_to_parent = Some(sender);
    }

    /// Spawn the listening task with the given callback
    pub(crate) fn spawn_listening_task<F, M>(&mut self, handler: F, mut rx: UnboundedReceiver<M>)
    where
        M: 'static,
        F: Fn(&mut Self, M) + 'static,
    {
        let this = self.deferred_borrow();
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(messages) = utils::recv_all(&mut rx).await {
                let Some(mut this) = this.borrow_mut() else {
                    break;
                };
                for message in messages {
                    handler(&mut this, message);
                }
            }
        });
    }

    /// Spawn a async task to recv messages from the parent
    pub(crate) fn spawn_recivier_task(&mut self, mut rx: UnboundedReceiver<T::ReceiveMessage>) {
        let this = self.deferred_borrow();
        wasm_bindgen_futures::spawn_local(async move {
            while let Some(messages) = utils::recv_all(&mut rx).await {
                let Some(mut this) = this.borrow_mut() else {
                    break;
                };
                for message in messages {
                    T::handle_message(&mut this, message);
                }
            }
        });
    }
}

/// Drop all children of the hook
fn drop_hook<T: Component>(ctx: &mut State<T>, hook: HookKey) {
    if let Some(hook) = ctx.hooks.remove(hook) {
        let mut hooks = hook.0.drop_us();
        for hook in hooks.drain(..) {
            drop_hook(ctx, hook);
        }
    }
}

/// Wrapper around a mutable state that only allows read-only access
///
/// This holds a mutable state to facilitate a few rendering features such as `.watch`
pub struct RenderCtx<'c, C: Component> {
    /// The inner context
    pub(crate) ctx: &'c mut State<C>,
    /// The render state for this state
    pub(crate) render_state: RenderingState<'c>,
}

impl<C: Component> Deref for RenderCtx<'_, C> {
    type Target = State<C>;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl<C: Component> RenderCtx<'_, C> {
    /// Calculate the value using the function and cache it using `clone`.
    /// Then whenever any signals read in the function are modified re-run the function and check
    /// if the new result is different.
    /// Only reruns the caller when the item is different.
    ///
    /// # Example
    /// ```rust
    /// # use natrix::prelude::*;
    /// # #[derive(Component)]
    /// # struct MyComponent {value: u32}
    /// #
    /// # impl Component for MyComponent {
    /// # type EmitMessage = NoMessages;
    /// # type ReceiveMessage = NoMessages;
    /// # fn render() -> impl Element<Self> {
    /// # |ctx: R<Self>| {
    /// if ctx.watch(|ctx| *ctx.value > 2) {
    ///     e::div().text(|ctx: R<Self>| *ctx.value)
    /// } else {
    ///     e::div().text("Value is too low")
    /// }
    /// # }}}
    /// ```
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
        let me = self.ctx.insert_hook(Box::new(hook));
        self.ctx.reg_dep(me);
        self.render_state.hooks.push(me);

        self.ctx.set_signals(signal_state);

        result
    }
}

/// The wather hook / signal
struct WatchState<F, T> {
    /// Function to calculate the state
    calc_value: F,
    /// The previous cached value
    last_value: T,
    /// The dependency that owns us.
    dep: HookKey,
}

impl<C, F, T> ReactiveHook<C> for WatchState<F, T>
where
    C: Component,
    T: PartialEq,
    F: Fn(&State<C>) -> T,
{
    fn update(&mut self, ctx: &mut State<C>, you: HookKey) -> UpdateResult {
        ctx.clear();
        let new_value = (self.calc_value)(ctx);
        ctx.reg_dep(you);

        if new_value == self.last_value {
            UpdateResult::Nothing
        } else {
            UpdateResult::RunHook(self.dep)
        }
    }

    fn drop_us(self: Box<Self>) -> Vec<HookKey> {
        Vec::new()
    }
}

/// This guard ensures that when it is in scope the data it was created for is `Some`
#[cfg_attr(feature = "nightly", must_not_suspend)]
#[derive(Clone, Copy)]
#[must_use]
pub struct Guard<F> {
    /// The closure for getting the value from a ctx
    getter: F,
}

/// Get a guard handle that can be used to retrieve the `Some` variant of a option without having to
/// use `.unwrap`.
/// Should be used to achieve find-grained reactivity (internally this uses `.watch` on `.is_some()`)
///
/// # Why?
/// The usecase can be seen by considering this logic:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value) = *ctx.value {
///     e::div().text(value)
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// The issue here is that the outer div (which might be a more expensive structure to create) is
/// recreated everytime `value` changes, even if it is `Some(0) -> Some(1)`
/// This is where you might reach for `ctx.watch`, and in fact that works perfectly:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if ctx.watch(|ctx| ctx.value.is_some()) {
///     e::div().text(|ctx: R<Self>| ctx.value.unwrap())
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// And this works, Now a change from `Some(0)` to `Some(1)` will only run the inner closure and
/// the outer div is reused. but there is one downside, we need `.unwrap` because the inner closure is
/// technically isolated, and this is ugly, and its easy to do by accident. and you might forget
/// the outer condition.
///
/// This is where guards come into play:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(|ctx| ctx.value.as_ref()) {
///     e::div().text(move |ctx: R<Self>| *ctx.get(&value_guard))
/// } else {
///     e::div().text("Is none")
/// }
/// # }}}
/// ```
/// Here `value_guard` is actually not the value at all, its a lightweight value thats can be
/// captured by child closures and basically is a way to say "I know that in this context this
/// value is `Some`"
///
/// Internally this uses `ctx.watch` and `.unwrap` (which should never fail)
///
/// ## Owned returns
/// By default this macro assumes the return value is `&T`, but if you want to return an owned
/// value you can use the `@owned` version:
/// ```rust
/// # use natrix::prelude::*;
/// # #[derive(Component)]
/// # struct MyComponent {value: Option<u32>}
/// # impl Component for MyComponent {
/// # type EmitMessage = NoMessages;
/// # type ReceiveMessage = NoMessages;
/// # fn render() -> impl Element<Self> {
/// # |ctx: R<Self>| {
/// if let Some(value_guard) = guard_option!(@owned |ctx| ctx.value) {
///    e::div().text(move |ctx: R<Self>| ctx.get_owned(&value_guard))
/// } else {
///    e::div().text("Is none")
/// }
/// # }}}
/// ```
#[macro_export]
macro_rules! guard_option {
    (| $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_some()) {
            Some(::natrix::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on None value")
            }))
        } else {
            None
        }
    };
    (@owned | $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_some()) {
            Some(::natrix::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect("Guard used on None value"),
            ))
        } else {
            None
        }
    };
}

/// Get a guard handle that can be used to retrieve the `Ok` variant of a option without having to
/// use `.unwrap`, or the `Err` variant.
#[macro_export]
macro_rules! guard_result {
    (| $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_ok()) {
            Ok(::natrix::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect("Guard used on Err value")
            }))
        } else {
            Err(::natrix::macro_ref::Guard::new::<Self, _>(move |$ctx| {
                $expr.expect_err("Guard used on Ok value")
            }))
        }
    };
    (@owned | $ctx:ident | $expr:expr) => {
        if $ctx.watch(move |$ctx| $expr.is_ok()) {
            Ok(::natrix::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect("Guard used on Err value"),
            ))
        } else {
            Err(::natrix::macro_ref::Guard::new_owned::<Self, _>(
                move |$ctx| $expr.expect_err("Guard used on Ok value"),
            ))
        }
    };
}

impl<F> Guard<F> {
    #[doc(hidden)]
    pub fn new<C, R>(getter: F) -> Self
    where
        F: for<'a> Fn(&'a State<C>) -> &'a R,
        C: Component,
    {
        Self { getter }
    }

    #[doc(hidden)]
    pub fn new_owned<C, R>(getter: F) -> Self
    where
        F: Fn(&State<C>) -> R,
        C: Component,
    {
        Self { getter }
    }
}

/// A combiend `Weak` and `RefCell` that facilities upgrading and borrowing as a shared
/// operation
#[must_use]
pub struct DeferredCtx<T: Component> {
    /// The `Weak<RefCell<T>>` in question
    inner: Weak<RefCell<State<T>>>,
}

// We put a bound on `'p` so that users are not able to store the upgraded reference (unless
// they want to use ouroboros themself to store it alongside the weak).
#[self_referencing]
struct DeferredRefInner<'p, T: Component> {
    rc: Rc<RefCell<State<T>>>,
    lifetime: PhantomData<&'p ()>,
    #[borrows(rc)]
    #[covariant]
    reference: RefMut<'this, State<T>>,
}

/// a `RefMut` that also holds a `Rc`.
/// See the `DeferredCtx::borrow_mut` on drop semantics and safety
#[cfg_attr(feature = "nightly", must_not_suspend)]
#[must_use]
pub struct DeferredRef<'p, T: Component>(DeferredRefInner<'p, T>);

impl<T: Component> DeferredCtx<T> {
    /// Borrow this `Weak<RefCell<...>>`, this will create a `Rc` for as long as the borrow is
    /// active. Returns `None` if the component was dropped. Its recommended to use the
    /// following construct to safely cancel async tasks:
    /// ```ignore
    /// let Some(mut borrow) = ctx.borrow_mut() else {return;};
    /// // ...
    /// drop(borrow);
    /// foo().await;
    /// let Some(mut borrow) = ctx.borrow_mut() else {return;};
    /// // ...
    /// };
    /// ```
    ///
    /// # Reactivity
    /// Calling this function clears the internal reactive flags (which is safe as long as the
    /// borrow safety rules below are followed).
    /// Once this value is dropped it will trigger a reactive update for any changed fields.
    ///
    /// # Borrow Safety
    /// The framework guarantees that it will never hold a borrow between event calls.
    /// This means the only source of panics is if you are holding a borrow when you yield to
    /// the event loop, i.e you should *NOT* hold this value across `.await` points.
    /// framework will regularly borrow the state on any registered event handler trigger, for
    /// example a user clicking a button.
    ///
    /// Keeping this type across an `.await` point or otherwise leading control to the event
    /// loop while the borrow is active could also lead to reactivity failrues and desyncs, and
    /// should be considered UB (not ub as in compile ub, but as in this framework makes no
    /// guarantees about what state the reactivity system will be in)
    ///
    /// ## Nightly
    /// The nightly feature flag enables a lint to detect this misuse.
    /// See the [Features]() chapther for details on how to set it up (it requires a bit more
    /// setup than just turning on the feature flag).
    #[cfg_attr(
        feature = "panic_hook",
        expect(
            clippy::missing_panics_doc,
            reason = "This happens when we already are in a panic"
        )
    )]
    #[must_use]
    pub fn borrow_mut(&self) -> Option<DeferredRef<'_, T>> {
        #[cfg(feature = "panic_hook")]
        assert!(!crate::panics::has_panicked());

        let rc = self.inner.upgrade()?;
        let borrow = DeferredRefInner::try_new(rc, PhantomData, |rc| rc.try_borrow_mut());

        let Ok(mut borrow) = borrow else {
            debug_assert!(
                false,
                "Deferred state borrowed while already borrowed. This might happen due to holding it across a yield point"
            );
            return None;
        };

        borrow.with_reference_mut(|ctx| ctx.clear());
        Some(DeferredRef(borrow))
    }
}

impl<T: Component> Deref for DeferredRef<'_, T> {
    type Target = State<T>;

    fn deref(&self) -> &Self::Target {
        self.0.borrow_reference()
    }
}
impl<T: Component> DerefMut for DeferredRef<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.with_reference_mut(|cell| &mut **cell)
    }
}

impl<T: Component> Drop for DeferredRef<'_, T> {
    fn drop(&mut self) {
        self.0.with_reference_mut(|ctx| {
            ctx.update();
        });
    }
}

impl<T: Component> State<T> {
    /// Get a wrapper around `Weak<RefCell<T>>` which provides a safer api that aligns with
    /// framework assumptions.
    pub fn deferred_borrow(&mut self) -> DeferredCtx<T> {
        DeferredCtx { inner: self.weak() }
    }

    /// Spawn a async task in the local event loop, which will run on the next possible moment.
    // This is `&mut` to make sure it cant be called in render callbacks.
    pub fn use_async<C, F>(&mut self, func: C)
    where
        C: FnOnce(DeferredCtx<T>) -> F,
        F: Future<Output = Option<()>> + 'static,
    {
        let deferred = self.deferred_borrow();
        let future = func(deferred);

        wasm_bindgen_futures::spawn_local(async {
            let _ = future.await;
        });
    }
}

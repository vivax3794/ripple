pub(crate) trait SmallAny {}
impl<T> SmallAny for T {}

use std::hash::Hash;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub(crate) struct RcCmpPtr<T>(pub Rc<T>);

impl<T> Clone for RcCmpPtr<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> PartialEq for RcCmpPtr<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for RcCmpPtr<T> {}

impl<T> Hash for RcCmpPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr_value = Rc::as_ptr(&self.0);
        ptr_value.hash(state);
    }
}

#[derive(Debug)]
pub(crate) struct WeakCmpPtr<T>(pub Weak<T>);

impl<T> Clone for WeakCmpPtr<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Weak::clone(&self.0))
    }
}

impl<T> PartialEq for WeakCmpPtr<T> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Eq for WeakCmpPtr<T> {}

impl<T> Hash for WeakCmpPtr<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let ptr_value = Weak::as_ptr(&self.0);
        ptr_value.hash(state);
    }
}

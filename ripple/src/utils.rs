use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug)]
pub struct RcCmpPtr<T>(pub Rc<T>);

impl<T> Clone for RcCmpPtr<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Deref for RcCmpPtr<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.0
            .deref()
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

#[cfg(not(unsafe_optimization))]
pub use std::cell::RefCell as InnerMut;

#[cfg(unsafe_optimization)]
#[derive(Default)]
pub struct InnerMut<T>(std::cell::UnsafeCell<T>);

#[cfg(unsafe_optimization)]
impl<T> InnerMut<T> {
    #[inline(always)]
    pub fn new(data: T) -> Self {
        Self(std::cell::UnsafeCell::new(data))
    }

    #[inline(always)]
    pub fn borrow(&self) -> &T {
        unsafe {
            &*self
                .0
                .get()
        }
    }

    #[allow(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn borrow_mut(&self) -> &mut T {
        unsafe {
            &mut *self
                .0
                .get()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[derive(Debug)]
    struct CantCompare;

    #[test]
    fn custom_rc_eq() {
        let x = RcCmpPtr(Rc::new(CantCompare));
        let y = x.clone();

        assert_eq!(x, y);
    }

    #[test]
    fn custom_rc_hashmap() {
        let x = RcCmpPtr(Rc::new(CantCompare));
        let y = x.clone();

        let mut map = HashMap::new();
        map.insert(y, 10);

        assert_eq!(map.get(&x), Some(&10));
    }

    #[test]
    fn inner_mut() {
        let x = InnerMut::new(0);

        assert_eq!(*x.borrow(), 0);
        *x.borrow_mut() = 10;
        assert_eq!(*x.borrow(), 10);
    }
}

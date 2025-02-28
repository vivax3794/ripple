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

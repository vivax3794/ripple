//! A spann holds source code locations.

use std::ops::{Deref, DerefMut};

/// Holds the location
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// byte offset
    start: usize,
    /// byte offset
    len: usize,
}

/// Holds a span and some data.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Spanned<T> {
    /// The span
    pub span: Span,
    /// The data
    pub data: T,
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for Spanned<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> From<Spanned<T>> for Span {
    fn from(spanned: Spanned<T>) -> Self {
        spanned.span
    }
}

impl<'a, T> From<&'a Spanned<T>> for &'a Span {
    fn from(spanned: &'a Spanned<T>) -> Self {
        &spanned.span
    }
}

impl Span {
    /// Creates a new span.
    #[must_use]
    pub const fn new_from_bounds(start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        let len = end.saturating_sub(start);
        Self { start, len }
    }

    /// Creates a new span
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self::new_from_bounds(start, end)
    }

    /// Creates a new span
    #[must_use]
    pub const fn new_from_len(start: usize, len: usize) -> Self {
        Self { start, len }
    }

    /// Returns true if the span is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns a `Spanned` from the span.
    pub fn spanned<T>(self, data: T) -> Spanned<T> {
        Spanned { span: self, data }
    }

    /// The end of the span
    pub fn end(&self) -> usize {
        self.start + self.len
    }

    /// Combines two spans.
    pub fn combine(self, other: Span) -> Span {
        let new_start = self.start.min(other.start);
        let new_end = self.end().max(other.end());
        Self::new_from_bounds(new_start, new_end)
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[test]
        fn converts_to_spanned(start: usize, len: usize, data: u8) {
            let span = Span::new_from_len(start, len);
            let spanned = span.spanned(data);

            assert_eq!(spanned.span, span);
            assert_eq!(spanned.data, data);
        }

        #[test]
        fn combine(start1: usize, len1: usize, start2: usize, len2: usize) {
            let span1 = Span::new_from_len(start1, len1);
            let span2 = Span::new_from_len(start2, len2);
            let span = span1.combine(span2);
            assert!(span.start <= span.end());
        }

        #[test]
        fn convert_to_span(start: usize, len: usize, data: u8) {
            let span = Span::new_from_bounds(start, len);
            let spanned = span.spanned(data);

            let new_span: Span = spanned.into();
            assert_eq!(new_span, span);
        }

        #[test]
        fn convert_to_span_ref(start: usize, len: usize, data: u8) {
            let span = Span::new_from_len(start, len);
            let spanned = span.spanned(data);

            let new_span: &Span = (&spanned).into();
            assert_eq!(*new_span, span);
        }

        #[test]
        fn not_empty(start: usize, len in 1usize..) {
            let span = Span::new_from_len(start, len);
            assert!(!span.is_empty());
        }

        #[test]
        fn empty(start: usize) {
            let span = Span::new_from_bounds(start, start);
            assert!(span.is_empty());
        }
    }
}

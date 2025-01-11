//! Generic holder of data with methods for peeking and reading.

#[cfg(test)]
use proptest_derive::Arbitrary;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Stream<T> {
    /// The data to be read
    data: Vec<T>,
    /// The current position in the data
    pos: usize,
}

impl<T> Stream<T> {
    /// Create a new stream
    pub fn new(data: Vec<T>) -> Stream<T> {
        Stream { data, pos: 0 }
    }

    /// Consume the next element in the stream
    #[cfg_attr(test, mutants::skip)] // This causes many tests to hang when mutated.
    #[must_use = "If you arent using the element use `skip` instead"]
    pub fn next(&mut self) -> Option<&T> {
        self.pos += 1;
        let element = self.data.get(self.pos - 1)?;
        Some(element)
    }

    /// Peek at the next element in the stream
    pub fn peek(&self) -> Option<&T> {
        self.peek_at(0)
    }

    /// Peek at a arbitrary element in the stream
    pub fn peek_at(&self, index: usize) -> Option<&T> {
        self.data.get(self.pos + index)
    }

    /// Progress the stream without emitting an element
    #[cfg_attr(test, mutants::skip)] // This causes many tests to hang when mutated.
    pub fn skip(&mut self) {
        self.pos += 1;
    }

    /// Go back in the stream
    pub fn regret(&mut self) {
        self.pos -= 1;
    }

    /// Get the current position in the stream
    pub fn pos(&self) -> usize {
        self.pos
    }

    /// The lenght of the stream
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn next() {
        let mut stream = Stream::new(vec![1, 2, 3]);
        assert_eq!(stream.next(), Some(&1));
        assert_eq!(stream.next(), Some(&2));
        assert_eq!(stream.next(), Some(&3));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn next_eof_progresses_stream() {
        let mut stream = Stream::new(vec![1]);
        let _ = stream.next();
        assert_eq!(stream.next(), None);
        stream.regret();
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn peek() {
        let mut stream = Stream::new(vec![1, 2, 3]);
        assert_eq!(stream.peek(), Some(&1));
        stream.skip();
        assert_eq!(stream.peek(), Some(&2));
        stream.skip();
        assert_eq!(stream.peek(), Some(&3));
        stream.skip();
        assert_eq!(stream.peek(), None);
    }

    #[test]
    fn peek_at() {
        let stream = Stream::new(vec![1, 2, 3]);
        assert_eq!(stream.peek_at(0), Some(&1));
        assert_eq!(stream.peek_at(1), Some(&2));
        assert_eq!(stream.peek_at(2), Some(&3));
        assert_eq!(stream.peek_at(3), None);
    }

    #[test]
    fn skip() {
        let mut stream = Stream::new(vec![1, 2, 3]);
        stream.skip();
        assert_eq!(stream.next(), Some(&2));
        stream.skip();
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn regret() {
        let mut stream = Stream::new(vec![1, 2, 3]);
        stream.skip();
        stream.regret();
        assert_eq!(stream.next(), Some(&1));
    }

    proptest! {
        #[test]
        fn next_peek(mut stream: Stream<u8>) {
            let peeked = stream.peek().copied();
            assert_eq!(stream.next(), peeked.as_ref());
        }
    }
}

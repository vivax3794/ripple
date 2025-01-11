//! Generic tokenizer methods for different langs

use crate::spann::{Span, Spanned};

/// Represents the result of a tokenization step, similar to an `Option`,
/// but with more specific `None`-like variants.
pub enum TokenizationResult<T> {
    /// Represents a successfully produced token to be added to the output.
    Token(T),
    /// Indicates that no token was produced, but tokenization should continue.
    /// Typically used for skipping whitespace or comments.
    Skip,
    /// Indicates that tokenization is complete and no further processing is needed.
    Done,
}

pub trait LangTokenizer<'c>
where
    Self: Sized,
{
    /// The kind of token produced.
    type Token;

    /// A general estimation of the ratio of input to tokens, this should be backed up by proper
    /// heuristics once the tokenizer is implemented and tested against real data.
    ///
    /// This is used to preallocate the output vector to avoid many resizes.
    const INPUT_TO_TOKEN_RATIO: usize;

    /// Produce the next token or indicate that tokenization is complete
    fn next_token(tokenizer: &mut Tokenizer<Self>) -> TokenizationResult<Self::Token>;
}

/// Tokenizer for css grammar
pub struct Tokenizer<'c, L> {
    /// The code to be tokenized
    code: &'c str,
    /// The byte position in the string
    position: usize,
    /// phantomdata
    _phantom: std::marker::PhantomData<L>,
}

impl<'c, T, L: LangTokenizer<'c, Token = T>> Tokenizer<'c, L> {
    /// Create a new tokenizer
    pub fn new(code: &'c str) -> Self {
        Self {
            code,
            position: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// The length of the next char
    fn next_char_len(&self) -> Option<usize> {
        Some(self.code.get(self.position..)?.chars().next()?.len_utf8())
    }

    /// Get a string slice of the next char, and progress the position
    pub fn next(&mut self) -> Option<char> {
        let result = self.code.get(self.position..)?.chars().next()?;
        self.position += result.len_utf8();
        Some(result)
    }

    /// Get a string slice of the next char
    pub fn peek(&self) -> Option<char> {
        self.code.get(self.position..)?.chars().next()
    }

    pub fn peek_at(&self, index: usize) -> Option<char> {
        self.code.get(self.position..)?.chars().nth(index)
    }

    /// Progress the stream without emitting an element
    pub fn skip(&mut self) {
        let len = self.next_char_len().unwrap_or(0);
        self.position += len;
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn slice(&self, start: usize, end: usize) -> &'c str {
        &self.code[start..end]
    }

    /// consume the inputs and return the tokens
    pub fn tokenize(mut self) -> Vec<Spanned<T>> {
        let mut tokens = Vec::with_capacity(self.code.len() / L::INPUT_TO_TOKEN_RATIO);
        loop {
            let start = self.position;
            match L::next_token(&mut self) {
                TokenizationResult::Token(token) => {
                    let span = Span::new_from_bounds(start, self.position);
                    tokens.push(span.spanned(token));
                }
                TokenizationResult::Skip => {}
                TokenizationResult::Done => break,
            }
        }
        tokens
    }

    /// Skip whitespace
    pub fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.position += c.len_utf8();
            } else {
                break;
            }
        }
    }

    /// Skip until the given str is found
    #[cfg_attr(test, mutants::skip)] // This causes many tests to hang when mutated.
    pub fn skip_until(&mut self, target: char) {
        while let Some(c) = self.next() {
            if c == target {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    struct TestLang;

    impl<'c> LangTokenizer<'c> for TestLang {
        type Token = char;
        const INPUT_TO_TOKEN_RATIO: usize = 1;
        fn next_token(tokenizer: &mut Tokenizer<Self>) -> TokenizationResult<char> {
            match tokenizer.next() {
                Some(c) => TokenizationResult::Token(c),
                None => TokenizationResult::Done,
            }
        }
    }

    #[test]
    fn tokenize() {
        let tokenizer = Tokenizer::<TestLang>::new("hello");
        let tokens: Vec<_> = tokenizer.tokenize().into_iter().map(|t| t.data).collect();
        assert_eq!(tokens, vec!['h', 'e', 'l', 'l', 'o']);
    }

    #[test]
    fn skip_whitespace() {
        let mut tokenizer = Tokenizer::<TestLang>::new("   hello");
        tokenizer.skip_whitespace();
        assert_eq!(tokenizer.next(), Some('h'));
    }

    #[test]
    fn skip_until() {
        let mut tokenizer = Tokenizer::<TestLang>::new("hello world");
        tokenizer.skip_until('w');
        assert_eq!(tokenizer.next(), Some('o'));
    }

    #[test]
    fn skip_until_not_found() {
        let mut tokenizer = Tokenizer::<TestLang>::new("hello");
        tokenizer.skip_until('x');
        assert_eq!(tokenizer.next(), None);
    }

    #[test]
    fn skip_whitespace_eof() {
        let mut tokenizer = Tokenizer::<TestLang>::new("");
        tokenizer.skip_whitespace();
        assert_eq!(tokenizer.next(), None);
    }

    proptest! {
        #[test]
        fn fuzzy(s: String) {
            let _ = Tokenizer::<TestLang>::new(&s);
        }

        #[test]
        fn proptest_skip_whitespace(s in "\\s") {
            let mut tokenizer = Tokenizer::<TestLang>::new(&s);
            tokenizer.skip_whitespace();
            assert_eq!(tokenizer.next(), None);
        }

        #[test]
        fn proptest_skip_non_whitespace(s in "\\S") {
            let mut tokenizer = Tokenizer::<TestLang>::new(&s);
            tokenizer.skip_whitespace();
            assert_eq!(tokenizer.position(), 0);
        }

        #[test]
        fn proptest_skip_until(s in "\\s*") {
            let mut tokenizer = Tokenizer::<TestLang>::new(&s);
            tokenizer.skip_until('x');
            assert_eq!(tokenizer.next(), None);
        }
    }
}

//! Css specific tokenizer
//!
//! https://drafts.csswg.org/css-syntax/#tokenizer-algorithms

use crate::spann::Spanned;
use crate::tokenizer::{LangTokenizer, TokenizationResult, Tokenizer};

enum TokenString<'c> {
    Direct(&'c str),
    ContainedEscapes(Box<str>),
}

/// A css token
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'c> {
    WhiteSpace,
    QuestionMark,
    Apostrophe,
    LeftParenthesis,
    RightParenthesis,
    Comma,
    Colon,
    Semicolon,
    LeftSquareBracket,
    RightSquareBracket,
    LeftCurlyBracket,
    RightCurlyBracket,
    Delimiter(char),
    String { text: &'c str, unclosed: bool },
    BadString { text: &'c str },
    HtmlCommentStart,
    Ident(&'c str),
    Function(&'c str),
    Url(&'c str),
}

impl<'c> Token<'c> {
    /// Was there a parse error when generating this token
    pub fn is_parse_error(&self) -> bool {
        matches!(
            self,
            Token::String { unclosed: true, .. } | Token::BadString { .. }
        )
    }
}

/// Tokenizer for css grammar
pub struct Css;

impl<'c> LangTokenizer<'c> for Css {
    type Token = Token<'c>;
    const INPUT_TO_TOKEN_RATIO: usize = 5;

    fn next_token(tokenizer: &mut Tokenizer<Self>) -> TokenizationResult<Token<'c>> {
        let Some(char) = tokenizer.next() else {
            return TokenizationResult::Done;
        };
        TokenizationResult::Token(match char {
            '?' => Token::QuestionMark,
            '\'' => Token::Apostrophe,
            '(' => Token::LeftParenthesis,
            ')' => Token::RightParenthesis,
            ',' => Token::Comma,
            ':' => Token::Colon,
            ';' => Token::Semicolon,
            '[' => Token::LeftSquareBracket,
            ']' => Token::RightSquareBracket,
            '{' => Token::LeftCurlyBracket,
            '}' => Token::RightCurlyBracket,
            '"' => handle_string(tokenizer, '"'),
            '/' if tokenizer.peek() == Some('*') => {
                handle_comment(tokenizer);
                return TokenizationResult::Skip;
            }
            '<' if tokenizer.peek() == Some('!')
                && tokenizer.peek_at(1) == Some('-')
                && tokenizer.peek_at(2) == Some('-') =>
            {
                tokenizer.skip();
                tokenizer.skip();
                tokenizer.skip();
                Token::HtmlCommentStart
            }
            _ if char.is_whitespace() => {
                let length = tokenizer.skip_whitespace();
                Token::WhiteSpace
            }
            _ if is_ident_start(char) => handle_ident(tokenizer, char),
            _ => Token::Delimiter(char),
        })
    }
}

fn handle_comment(tokenizer: &mut Tokenizer<Css>) {
    tokenizer.skip();
    while tokenizer.peek().is_some() {
        tokenizer.skip_until('*');
        if tokenizer.peek() == Some('/') {
            tokenizer.skip();
            return;
        }
    }
}

fn handle_string<'c>(tokenizer: &mut Tokenizer<'c, Css>, starting_char: char) -> Token<'c> {
    let start = tokenizer.position();
    let mut unclosed = false;

    loop {
        let c = tokenizer.next();
        match c {
            Some('\n') => {
                tokenizer.code.regret();
                return tokenizer.spanned_token(
                    Token::BadString {
                        text: string.into_boxed_str(),
                    },
                    count,
                );
            }
            Some('\\') => {
                count += 1;
                if tokenizer.code.peek().is_none() {
                } else if tokenizer.code.peek() == Some(&'\n') {
                    tokenizer.code.skip();
                    count += 1;
                } else {
                    let (result, length) = consume_escape_sequence(tokenizer);
                    string.push(result);
                    count += length;
                }
            }
            Some(c) if c == starting_char => {
                break;
            }
            Some(c) => {}
            None => {
                unclosed = true;
                break;
            }
        }
    }
    Token::String {
        text: string.into_boxed_str(),
        unclosed,
    }
}

fn consume_escape_sequence(tokenizer: &mut Tokenizer<Css>) -> (char, usize) {
    if let Some(c) = tokenizer.code.next() {
        match c {
            c if c.is_ascii_hexdigit() => {
                let mut count = 0;
                let mut hex = String::new();
                hex.push(*c);
                count += 1;
                while tokenizer.code.peek().is_some_and(|c| c.is_ascii_hexdigit()) {
                    hex.push(*tokenizer.code.next().unwrap());
                    count += 1;
                    if hex.len() == 6 {
                        break;
                    }
                }
                if tokenizer.code.peek().is_some_and(|c| c.is_whitespace()) {
                    tokenizer.code.skip();
                    count += 1;
                }
                let resutl = std::char::from_u32(u32::from_str_radix(&hex, 16).unwrap())
                    .unwrap_or('\u{FFFD}');
                (resutl, count)
            }
            c => (*c, 1),
        }
    } else {
        ('\u{FFFD}', 1)
    }
}

/// https://drafts.csswg.org/css-syntax/#ident-start-code-point
fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic()
        || matches!(c,
            '_' | '\u{00B7}' | '\u{00C0}'..='\u{00D6}' | '\u{00D8}'..='\u{00F6}' | '\u{00F8}'..='\u{037D}' | '\u{037F}'..='\u{1FFF}' | '\u{200C}' | '\u{200D}' | '\u{203F}' | '\u{2040}' | '\u{2070}'..='\u{218F}' | '\u{2C00}'..='\u{2FEF}' | '\u{3001}'..='\u{D7FF}' | '\u{F900}'..='\u{FDCF}' | '\u{FDF0}'..='\u{FFFD}' | '\u{10000}'..
        )
}

/// https://drafts.csswg.org/css-syntax/#ident-code-point
fn is_ident_char(c: char) -> bool {
    is_ident_start(c) || c.is_ascii_digit() || c == '-'
}

fn handle_ident(tokenizer: &mut CssTokenizer) -> Spanned<Token> {
    let content = handle_ident_string(tokenizer);
    let mut span = content.span;

    let token = if tokenizer.code.peek() == Some(&'(') {
        tokenizer.code.skip();
        span.end += 1;
        if content.data.eq_ignore_ascii_case("url") {
            while tokenizer.code.peek().is_some_and(|c| c.is_whitespace())
                && tokenizer.code.peek_at(1).is_some_and(|c| c.is_whitespace())
            {
                span.end += 1;
                tokenizer.code.skip();
            }
            if tokenizer
                .code
                .peek()
                .is_some_and(|&c| c == '\'' || c == '"')
                || tokenizer
                    .code
                    .peek_at(1)
                    .is_some_and(|&c| c == '\'' || c == '"')
            {
                Token::Function(content.data.into())
            } else {
                let (result, length) = handle_url(tokenizer);
                span.end += length;
                result
            }
        } else {
            Token::Function(content.data.into())
        }
    } else {
        Token::Ident(content.data.into())
    };

    span.spanned(token)
}

fn handle_ident_string(tokenizer: &mut CssTokenizer) -> Spanned<String> {
    let mut result = String::new();
    let mut length = 0;
    loop {
        match tokenizer.code.next().copied() {
            Some(c) if is_ident_char(c) => {
                result.push(c);
                length += 1;
            }
            Some(c) if is_valid_escape_start(c, *tokenizer.code.peek().unwrap_or(&'\0')) => {
                length += 1;
                let (char, escape_length) = consume_escape_sequence(tokenizer);
                result.push(char);
                length += escape_length;
            }
            _ => {
                tokenizer.code.regret();
                break;
            }
        }
    }

    tokenizer.code.span_length(length).spanned(result)
}

fn handle_url(tokenizer: &mut CssTokenizer) -> (Token, usize) {
    todo!()
}

fn is_valid_escape_start(first: char, second: char) -> bool {
    first == '\\' && second != '\n'
}

/// Tokenizer for css
pub type CssTokenizer = Tokenizer<Css>;

#[cfg(test)]
mod tests {

    use proptest::prelude::*;
    use yare::parameterized;

    use super::*;
    use crate::spann::Span;

    #[parameterized(
        question_mark = {"?", &[Token::QuestionMark]},
        apostrophe = {"'", &[Token::Apostrophe]},
        left_parenthesis = {"(", &[Token::LeftParenthesis]},
        right_parenthesis = {")", &[Token::RightParenthesis]},
        comma = {",", &[Token::Comma]},
        colon = {":", &[Token::Colon]},
        semicolon = {";", &[Token::Semicolon]},
        left_square_bracket = {"[", &[Token::LeftSquareBracket]},
        right_square_bracket = {"]", &[Token::RightSquareBracket]},
        left_curly_bracket = {"{", &[Token::LeftCurlyBracket]},
        right_curly_bracket = {"}", &[Token::RightCurlyBracket]},
        double_question_mark = {"??", &[Token::QuestionMark, Token::QuestionMark]},
        question_parenthesis = {"?(", &[Token::QuestionMark, Token::LeftParenthesis]},
        question_comma = {"?,", &[Token::QuestionMark, Token::Comma]},
        full_parenthesis = {"()", &[Token::LeftParenthesis, Token::RightParenthesis]},
        comma_paren = {",)", &[Token::Comma, Token::RightParenthesis]},
        whitespace_single = {" ", &[Token::WhiteSpace]},
        whitespace_double = {"  ", &[Token::WhiteSpace]},
        whitespace_with_question_mark = {" ? ", &[Token::WhiteSpace, Token::QuestionMark, Token::WhiteSpace]},
        comment = {"/* hello */", &[]},
        double_comment = {"/* hello *//* world */", &[]},
        comment_with_star = {"/* 1 * 2 */", &[]},
        comment_is_ignored = {"?/* hello */?", &[Token::QuestionMark, Token::QuestionMark]},
        unclosed_comment = {"/* hello", &[Token::UnclosedComment]},
        string = {"\"hello\"", &[Token::String{ text: "hello".into(), unclosed: false }]},
        escape_non_hex = {"\"\\n\"", &[Token::String { text: "n".into(), unclosed: false }]},
        escape_hex = {"\"\\A\"", &[Token::String { text: "\n".into(), unclosed: false }]},
        escape_hex_overflow = {"\"\\00000AAA\"", &[Token::String { text: "\nAA".into(), unclosed: false }]},
        escape_newline = {"\"a\\\nb\"", &[Token::String { text: "ab".into(), unclosed: false }]},
        bad_string = {"\"hello\n", &[Token::BadString { text: "hello".into() }, Token::WhiteSpace]},
        escape_at_eof = {"\"hello\\", &[Token::String { text: "hello".into(), unclosed: true }]},
        escape_apostrophe = {"\"a\\\"b\"", &[Token::String { text: "a\"b".into(), unclosed: false }]},
        espace_into_whitespace = {"\"a\\A \"", &[Token::String { text: "a\n".into(), unclosed: false }]},
        html_comment_start = {"<!--", &[Token::HtmlCommentStart]},
        ident = {"hello", &[Token::Ident("hello".into())]},
        ident_with_number = {"hello1", &[Token::Ident("hello1".into())]},
        ident_unicode = {"héllo", &[Token::Ident("héllo".into())]},
        ident_escape_unicode = {"h\\65llo", &[Token::Ident("h\u{65}llo".into())]},
        function = {"hello(", &[Token::Function("hello".into())]},
    )]
    fn parse_results(code: &str, expected: &[Token]) {
        let tokenizer = CssTokenizer::new(code.to_string());
        let tokens = tokenizer.tokenize();
        let tokens = tokens.into_iter().map(|t| t.data).collect::<Vec<_>>();
        assert_eq!(tokens, expected);
    }

    #[parameterized(
        question_mark = {"?", &[Span::new(0, 1)]},
        double_question_mark = {"??", &[Span::new(0, 1), Span::new(1, 2)]},
        full_parenthesis = {"()", &[Span::new(0, 1), Span::new(1, 2)]},
        whitespace_single = {" ", &[Span::new(0, 1)]},
        whitespace_double = {"  ", &[Span::new(0, 2)]},
        whitespace_with_question_mark = {" ? ", &[Span::new(0, 1), Span::new(1, 2), Span::new(2, 3)]},
        unclosed_comment = {"?/* a", &[Span::new(0, 1), Span::new(6, 6)]},
        comment_is_ignored = {"?/*a*/?", &[Span::new(0, 1), Span::new(6, 7)]},
        string = {"\"hello\"", &[Span::new(0, 7)]},
        string_escape = {"\"a\\nb\"", &[Span::new(0, 6)]},
        string_escape_newline = {"\"a\\\nb\"", &[Span::new(0, 6)]},
        string_late = {"/*aaa*/\"hello\"", &[Span::new(7, 14)]},
        string_escape_late = {"/*aaa*/\"ab\\nbc\"", &[Span::new(7, 15)]},
        string_escape_newline_late = {"/*aaa*/\"ab\\\nbc\"", &[Span::new(7, 15)]},
        string_escape_hex_late = {"/*aaa*/\"ab\\AA\"", &[Span::new(7, 14)]},
        string_escape_hex_whitespace_late = {"/*aaa*/\"ab\\A A\"", &[Span::new(7, 15)]},
        ident = {"hello", &[Span::new(0, 5)]},
        ident_unicode = {"héllo", &[Span::new(0, 5)]},
        ident_escape_unicode = {"he\\65llo", &[Span::new(0, 8)]},
        function = {"hello(", &[Span::new(0, 6)]},
    )]
    fn parse_spans(code: &str, expected: &[Span]) {
        let tokenizer = CssTokenizer::new(code.to_string());
        let tokens = tokenizer.tokenize();
        let tokens = tokens.into_iter().map(|t| t.span).collect::<Vec<_>>();
        assert_eq!(tokens, expected);
    }

    #[parameterized(
        unclosed_comment = {"/* hello"},
        unclosed_string_double = {"\"hello"},
        bad_string = {"\"hello\n"},
    )]
    fn is_parse_error(code: &str) {
        let tokenizer = CssTokenizer::new(code.to_string());
        let tokens = tokenizer.tokenize();
        assert!(tokens.into_iter().any(|t| t.data.is_parse_error()));
    }

    proptest! {
        #[test]
        fn fuzzy(code: String) {
            let tokenizer = CssTokenizer::new(code);
            let _ = tokenizer.tokenize();
        }

        #[test]
        fn pure_whitespace(s in "\\s+") {
            let tokenizer = CssTokenizer::new(s.to_string());
            let tokens = tokenizer.tokenize();
            assert_eq!(tokens.len(), 1);
            assert_eq!(tokens[0].data, Token::WhiteSpace);
        }

        #[test]
        fn whitespace_with_non_whitespace(s in "\\s+\\S") {
            let tokenizer = CssTokenizer::new(s.to_string());
            let tokens = tokenizer.tokenize();
            assert!(tokens.len() > 1);
            assert_eq!(tokens[0].data, Token::WhiteSpace);
        }

        #[test]
        fn no_duplicate_whitespace(s: String) {
            let tokenizer = CssTokenizer::new(s);
            let tokens = tokenizer.tokenize();
            for i in 0..(tokens.len().saturating_sub(1)) {
               if tokens[i].data == Token::WhiteSpace && tokens[i+1].data == Token::WhiteSpace {
                   panic!("duplicate whitespace");
               }
            }
        }

        #[test]
        fn comment(s in "/\\*[^/]*\\*/") {
            let tokenizer = CssTokenizer::new(s.to_string());
            let tokens = tokenizer.tokenize();
            assert!(tokens.is_empty());
        }

        #[test]
        fn string(s in r#""[^\"\n]*""#) {
            let tokenizer = CssTokenizer::new(s.to_string());
            let tokens = tokenizer.tokenize();
            assert_eq!(tokens.len(), 1);
        }
    }
}

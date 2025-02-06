//! This is the html and css parser for the ripple project.
pub use lazy_static::lazy_static;
use thiserror::Error;
pub use tree_sitter;
use tree_sitter::{Node, Query, QueryCursor, QueryMatches, TextProvider};

lazy_static! {
    pub static ref CSS: tree_sitter::Language = tree_sitter_css::LANGUAGE.into();
}

#[macro_export]
macro_rules! query {
    ($lang:ident : $name:ident = $text:literal) => {
        ::ripple_parser::lazy_static! {
            static ref $name: ::ripple_parser::tree_sitter::Query =
                ::ripple_parser::tree_sitter::Query::new(&::ripple_parser::$lang, $text).unwrap();
        }
    };
}

pub type RawDocument = ropey::Rope;

/// Errors during parsing
#[derive(Debug, Error, Clone, Copy)]
pub enum ParserError {
    /// The parser timed out.
    ///
    /// Techically speaking this is also triggered if the parser is explicitly canceled, or the
    /// language wasnt set. neither of which this wrapper does (hence the specific variant name)
    #[error("Parser timedout")]
    ParserTimedout,
}

/// Create a css parser
///
/// # Errors
/// If it couldnt load the parser
pub fn construct_css_parser() -> Result<tree_sitter::Parser, tree_sitter::LanguageError> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_css::LANGUAGE.into();
    parser.set_language(&language)?;
    Ok(parser)
}

/// A css document
pub struct Document {
    /// The parser for this file
    parser: tree_sitter::Parser,
    /// The tree for this file
    pub tree: tree_sitter::Tree,
    /// Raw document text
    pub document: ropey::Rope,
}

impl Document {
    /// Create a new document from file contents
    ///
    /// # Errors
    /// If failed to load language for parsing, or if parsing times out
    pub fn parse(content: String, mut parser: tree_sitter::Parser) -> Result<Self, ParserError> {
        let tree = parser
            .parse(&content, None)
            .ok_or(ParserError::ParserTimedout)?;
        Ok(Self {
            parser,
            tree,
            document: ropey::Rope::from(content),
        })
    }

    /// Fully replaces the document.
    pub fn replace_document(&mut self, new_text: String) {
        let end_line = self.document.len_lines();
        let end_char = self.document.line(end_line - 1).len_chars();
        self.apply_edit(Edit {
            start_line: 0,
            start_collumn: 0,
            end_line,
            end_collumn: end_char,
            new_text,
        });
    }

    fn char_index(&self, line: usize, collumn: usize) -> usize {
        self.document.line_to_char(line) + collumn
    }

    /// Apply a edit to the document and the tree spans
    pub fn apply_edit(&mut self, edit: Edit) {
        let start_char = self.char_index(edit.start_line, edit.start_collumn);
        let end_char = self.char_index(edit.end_line, edit.end_collumn);
        let start_byte = self.document.char_to_byte(start_char);
        let end_byte = self.document.char_to_byte(end_char);

        self.document.remove(start_char..end_char);
        self.document.insert(start_char, &edit.new_text);

        let new_end_byte = start_byte + edit.new_text.len();
        let new_end_line = self.document.byte_to_line(new_end_byte);
        let new_end_collumn =
            self.document.byte_to_char(new_end_byte) - self.document.line_to_char(new_end_line);

        let ts_edit = tree_sitter::InputEdit {
            start_byte,
            start_position: tree_sitter::Point {
                row: edit.start_line,
                column: edit.start_collumn,
            },
            old_end_byte: end_byte,
            new_end_byte,
            old_end_position: tree_sitter::Point {
                row: edit.end_line,
                column: edit.end_collumn,
            },
            new_end_position: tree_sitter::Point {
                row: new_end_line,
                column: new_end_collumn,
            },
        };
        self.tree.edit(&ts_edit);
    }

    /// Reparse the tree
    pub fn reparse(&mut self) {
        self.tree = self
            .parser
            .parse_with_options(
                // Rope lets us "fake" having continious string, but with good editing!
                &mut |byte, _| get_chunk_starting_byte(byte, &self.document),
                Some(&self.tree),
                None,
            )
            .unwrap();
    }

    pub fn match_query<'s, 'q>(
        &'s self,
        query: &'q Query,
        cursor: &'q mut QueryCursor,
    ) -> QueryMatches<'q, 's, RopeTextProvider<'s>, &'s [u8]> {
        cursor.matches(
            query,
            self.tree.root_node(),
            RopeTextProvider(&self.document),
        )
    }

    pub fn get_text(&self, node: Node) -> String {
        self.document.byte_slice(node.byte_range()).to_string()
    }
}

fn get_chunk_starting_byte(byte: usize, document: &ropey::Rope) -> &str {
    let (text, chunk_begin_byte, _chunk_begin_char, _chunk_begin_line) =
        document.chunk_at_byte(byte);
    let offset_into_chunk = byte - chunk_begin_byte;
    &text[offset_into_chunk..]
}

pub struct RopeTextProvider<'r>(&'r ropey::Rope);

impl<'r> TextProvider<&'r [u8]> for RopeTextProvider<'r> {
    type I = RopeChunksBytes<'r>;

    fn text(&mut self, node: Node) -> Self::I {
        RopeChunksBytes(self.0.byte_slice(node.byte_range()).chunks())
    }
}

pub struct RopeChunksBytes<'r>(ropey::iter::Chunks<'r>);

impl<'r> Iterator for RopeChunksBytes<'r> {
    type Item = &'r [u8];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|chunk| chunk.as_bytes())
    }
}

/// A edit to a document
pub struct Edit {
    pub start_line: usize,
    pub start_collumn: usize,
    pub end_line: usize,
    pub end_collumn: usize,
    pub new_text: String,
}

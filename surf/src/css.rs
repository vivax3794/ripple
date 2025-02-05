use std::collections::HashSet;

use ripple_parser::tree_sitter::StreamingIterator;
use ripple_parser::{lazy_static, query_css};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag};

use crate::{Language, node_range};

#[derive(Default)]
pub struct Css {}

// SYNTAX_ERRORS: stuff that TS sees as syntax errors
// SPECIFIC_ERRORS: Patterns matching specific patterns of (ERROR) to provide better messages

query_css!(
    SYNTAX_ERRORS = r#"
    (MISSING) @missing-node
    (ERROR) @error-node
"#
);
query_css!(
    SPECIFIC_ERRORS = r#"
    (
        (ERROR "\#") @error-node
        (integer_value) @invalid-color
    )
"#
);
query_css!(
    OTHER_QUERIES = r#"
    (
        (color_value) @color
        (#not-match? @color "^\#(.{3,4}|.{6}|.{8})$")
    )
    (block
        (declaration (property_name) @name_1) @overwritten-decl
        (declaration (property_name) @name_2) @shadowing-decl
        (#eq? @name_1 @name_2)
    )
    (integer_value
        (unit) @unit
    ) @integer
    "#
);

// https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Values_and_Units#numeric_data_types
lazy_static! {
    static ref LENGTH_UNITS: HashSet<&'static str> = HashSet::from([
        "cap", "ch", "em", "ex", "ic", "lh", "rcap", "rch", "rem", "rec", "ric", "rlh", "dvh",
        "dvw", "lvh", "svh", "vb", "vh", "vi", "vmax", "vmin", "vw", "cqb", "cqh", "cqi", "cqmax",
        "cqmin", "cqw", "cm", "in", "mm", "pc", "pt", "px", "Q"
    ]);
}
lazy_static! {
    static ref OTHER_UNITS: HashSet<&'static str> = HashSet::from([
        "deg", "grad", "rad", "turn", "ms", "s", "hz", "khz", "fr", "dpcm", "dpi", "dppx", "x",
        "%",
    ]);
}

impl Language for Css {
    fn diagnostics(
        &self,
        document: &mut ripple_parser::Document,
    ) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        let mut result = Vec::new();

        let handled_errors = specific_errors(document, &mut result);
        syntax_errors(document, &mut result, handled_errors);
        other_diagnostics(document, &mut result);

        result
    }
}

fn other_diagnostics(document: &mut ripple_parser::Document, result: &mut Vec<Diagnostic>) {
    let (mut matches, document) = document.match_query(&OTHER_QUERIES);
    while let Some(query_match) = matches.next() {
        match query_match.pattern_index {
            0 => {
                let color_node = query_match.captures[0].node;
                let range = node_range(&color_node);
                let len = color_node.byte_range().len() - 1;
                result.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Hex color must be 3,4,6 or 8 charcthers. found {len}"),
                    ..Diagnostic::default()
                });
            }
            1 => {
                //let name_1 = query_match.captures[1].node;
                let name_1_decl = query_match.captures[0].node;
                //let name_2 = query_match.captures[3].node;
                let name_2_decl = query_match.captures[2].node;

                result.push(Diagnostic {
                    range: node_range(&name_1_decl),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: "Property is shadowed later in block".to_string(),
                    tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                    ..Diagnostic::default()
                });
                result.push(Diagnostic {
                    range: node_range(&name_2_decl),
                    severity: Some(DiagnosticSeverity::HINT),
                    message: "Property is overshadowed here".to_string(),
                    ..Diagnostic::default()
                });
            }
            2 => {
                let unit_node = query_match.captures[1].node;
                let int_node = query_match.captures[0].node;

                let unit = document
                    .byte_slice(unit_node.byte_range())
                    .to_string()
                    .to_lowercase();

                let int_start = int_node.start_byte();
                let int_end = unit_node.start_byte();
                let int_value = document.byte_slice(int_start..int_end);

                if LENGTH_UNITS.contains(unit.as_str()) {
                    if int_value == "0" {
                        result.push(Diagnostic {
                            range: node_range(&unit_node),
                            severity: Some(DiagnosticSeverity::WARNING),
                            message: "Units not needed for 0 <length> value".to_string(),
                            tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                            ..Diagnostic::default()
                        });
                    }
                } else if !OTHER_UNITS.contains(unit.as_str()) {
                    result.push(Diagnostic {
                        range: node_range(&unit_node),
                        severity: Some(DiagnosticSeverity::ERROR),
                        message: format!("{unit} is not a valid unit"),
                        ..Diagnostic::default()
                    });
                }
            }
            _ => unreachable!(),
        }
    }
}

fn syntax_errors(
    document: &mut ripple_parser::Document,
    result: &mut Vec<Diagnostic>,
    handled_errors: Vec<usize>,
) {
    let (mut matches, _) = document.match_query(&SYNTAX_ERRORS);
    while let Some(query_match) = matches.next() {
        let error_node = query_match.captures[0].node;

        let message = match query_match.pattern_index {
            0 => format!("Expected {}", error_node.grammar_name()),
            1 => {
                if handled_errors.contains(&error_node.id()) {
                    continue;
                }
                "Syntax Error".to_owned()
            }
            _ => unreachable!(),
        };

        result.push(Diagnostic {
            range: node_range(&error_node),
            severity: Some(DiagnosticSeverity::ERROR),
            message,
            ..Diagnostic::default()
        });
    }
}

fn specific_errors(
    document: &mut ripple_parser::Document,
    result: &mut Vec<Diagnostic>,
) -> Vec<usize> {
    let mut handled_errors = Vec::new();
    let (mut matches, _) = document.match_query(&SPECIFIC_ERRORS);
    while let Some(query_match) = matches.next() {
        let (message, range, error_node) = match query_match.pattern_index {
            0 => {
                let int_node = query_match.captures[1].node;
                let range = node_range(&int_node);
                let len = int_node.byte_range().len();
                (
                    format!("Hex color must be 3,4,6 or 8 charcthers. found {len}"),
                    range,
                    query_match.captures[0].node,
                )
            }
            _ => unreachable!(),
        };

        handled_errors.push(error_node.id());

        result.push(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::ERROR),
            message,
            ..Diagnostic::default()
        });
    }
    handled_errors
}

#[cfg(test)]
mod tests {
    use ripple_parser::tree_sitter::Query;

    use super::{OTHER_QUERIES, SPECIFIC_ERRORS, SYNTAX_ERRORS};

    // If the queries are invalid they will panic when used,
    // and debugging a lsp is kinda hell because its the editor that spawns it.
    // this test makes sure all the queries are valid!
    #[test]
    fn load_queries() {
        let _: &Query = &SYNTAX_ERRORS;
        let _: &Query = &SPECIFIC_ERRORS;
        let _: &Query = &OTHER_QUERIES;
    }
}

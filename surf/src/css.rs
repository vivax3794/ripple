use std::thread;

use lazy_static::lazy_static;
use ripple_parser::query;
use ripple_parser::tree_sitter::StreamingIterator;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, DiagnosticTag, NumberOrString};

use crate::lsp::{Language, node_range};

#[derive(Default)]
pub struct Css {}

query!(CSS:
    SYNTAX_ERRORS = r#"
    (MISSING) @missing-node
    (ERROR) @error-node
    "#
);
query!(CSS:
    COLOR_NODES = r#"
    (color_value) @color
    "#
);
query!(CSS:
    DECLARATION_NAMES = r#"
    (block
        (declaration (property_name) @name)*
    )
    "#
);
query!(CSS:
    INTEGERS = r#"
    (integer_value
        (unit) @unit
    ) @integer
    "#
);

// https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_Values_and_Units#numeric_data_types
lazy_static! {
    static ref LENGTH_UNITS: HashSet<&'static str> = HashSet::from_iter([
        "cap", "ch", "em", "ex", "ic", "lh", "rcap", "rch", "rem", "rec", "ric", "rlh", "dvh",
        "dvw", "lvh", "svh", "vb", "vh", "vi", "vmax", "vmin", "vw", "cqb", "cqh", "cqi", "cqmax",
        "cqmin", "cqw", "cm", "in", "mm", "pc", "pt", "px", "Q"
    ]);
}
lazy_static! {
    static ref OTHER_UNITS: HashSet<&'static str> = HashSet::from_iter([
        "deg", "grad", "rad", "turn", "ms", "s", "hz", "khz", "fr", "dpcm", "dpi", "dppx", "x",
        "%",
    ]);
}

impl Language for Css {
    fn diagnostics(
        &self,
        document: &ripple_parser::Document,
    ) -> Vec<tower_lsp::lsp_types::Diagnostic> {
        thread::scope(|scope| {
            let syntax_errors = scope.spawn(|| syntax_errors(document));
            let colors = scope.spawn(|| colors(document));
            let units = scope.spawn(|| handle_units(document));
            let blocks = scope.spawn(|| handle_block_properties(document));

            let mut result = Vec::new();
            result.append(&mut syntax_errors.join().unwrap());
            result.append(&mut colors.join().unwrap());
            result.append(&mut units.join().unwrap());
            result.append(&mut blocks.join().unwrap());
            result
        })
    }
}

#[inline(always)]
fn syntax_errors(document: &ripple_parser::Document) -> Vec<Diagnostic> {
    let mut cursor = ripple_parser::tree_sitter::QueryCursor::new();
    let mut result = Vec::new();
    document
        .match_query(&SYNTAX_ERRORS, &mut cursor)
        .for_each(|query_match| match query_match.pattern_index {
            0 => {
                let node = query_match.captures[0].node;
                let name = node.grammar_name();
                let range = node_range(&node);
                result.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Expected {name}"),
                    code: Some(NumberOrString::String("css_invalid_syntax".to_string())),
                    ..Diagnostic::default()
                });
            }
            1 => {
                let node = query_match.captures[0].node;
                let range = node_range(&node);
                result.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: "Syntax Error".to_string(),
                    code: Some(NumberOrString::String("css_invalid_syntax".to_string())),
                    ..Diagnostic::default()
                });
            }
            _ => unreachable!(),
        });
    result
}

fn colors(document: &ripple_parser::Document) -> Vec<Diagnostic> {
    let mut cursor = ripple_parser::tree_sitter::QueryCursor::new();
    let mut result = Vec::new();
    document
        .match_query(&COLOR_NODES, &mut cursor)
        .for_each(|query_match| {
            let color_node = query_match.captures[0].node;
            let range = node_range(&color_node);
            let len = color_node.byte_range().len() - 1;

            if !matches!(len, 3 | 4 | 6 | 8) {
                result.push(Diagnostic {
                    range,
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Hex color must be 3,4,6 or 8 charcthers. found {len}"),
                    code: Some(NumberOrString::String("css_invalid_hex".to_string())),
                    ..Diagnostic::default()
                });
            }
        });
    result
}

fn handle_units(document: &ripple_parser::Document) -> Vec<Diagnostic> {
    let mut cursor = ripple_parser::tree_sitter::QueryCursor::new();
    let mut result = Vec::new();
    document
        .match_query(&INTEGERS, &mut cursor)
        .for_each(|query_match| {
            let unit_node = query_match.captures[1].node;
            let int_node = query_match.captures[0].node;

            let unit = document.get_text(unit_node).to_lowercase();

            let int_start = int_node.start_byte();
            let int_end = unit_node.start_byte();
            let int_value = document.document.byte_slice(int_start..int_end);

            if LENGTH_UNITS.contains(unit.as_str()) {
                if int_value == "0" {
                    result.push(Diagnostic {
                        range: node_range(&unit_node),
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: "Units not needed for 0 <length> value".to_string(),
                        tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                        code: Some(NumberOrString::String("css_zero_unit".to_string())),
                        ..Diagnostic::default()
                    });
                }
            } else if !OTHER_UNITS.contains(unit.as_str()) {
                result.push(Diagnostic {
                    range: node_range(&unit_node),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("{unit} is not a valid unit"),
                    code: Some(NumberOrString::String("css_invalid_unit".to_string())),
                    ..Diagnostic::default()
                });
            }
        });
    result
}

fn handle_block_properties(document: &ripple_parser::Document) -> Vec<Diagnostic> {
    let mut cursor = ripple_parser::tree_sitter::QueryCursor::new();
    let mut result = Vec::new();
    document
        .match_query(&DECLARATION_NAMES, &mut cursor)
        .for_each(|query_match| {
            let mut properties =
                HashMap::with_capacity_and_hasher(query_match.captures.len(), Default::default());
            for capture in query_match.captures.iter().rev() {
                let node = capture.node;
                let name = document.get_text(node);
                let range = node_range(&node.parent().unwrap_or(node));

                if let Err(mut entry) = properties.try_insert(name, (range, false)) {
                    result.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::WARNING),
                        message: "Property is shadowed later in block".to_string(),
                        tags: Some(vec![DiagnosticTag::UNNECESSARY]),
                        code: Some(NumberOrString::String("css_overshadowed".to_string())),
                        ..Diagnostic::default()
                    });

                    let current_value = entry.entry.get_mut();
                    if !current_value.1 {
                        current_value.1 = true;
                        result.push(Diagnostic {
                            range: current_value.0,
                            severity: Some(DiagnosticSeverity::HINT),
                            message: "Property is overshadowed here".to_string(),
                            code: Some(NumberOrString::String("css_overshadowed_hint".to_string())),
                            ..Diagnostic::default()
                        });
                    }
                }
            }
        });
    result
}

#[cfg(test)]
mod tests {
    use ripple_parser::construct_css_parser;
    use yare::parameterized;

    use super::*;

    /// Runs diagnostics on the given code and returns a vec of diagnostics codes created
    /// (We dont bother really checking range, etc as it gets very verbose)
    /// (and the actually important part is a lint triggering or not)
    fn run_diagnostics(code: &'static str) -> Vec<String> {
        let parser = ripple_parser::construct_css_parser().unwrap();
        let document = ripple_parser::Document::parse(code.to_owned(), parser).unwrap();
        let expr = document.tree.root_node().to_sexp();
        println!("{expr}");
        let diagnsotics = Css::default().diagnostics(&document);
        diagnsotics
            .into_iter()
            .map(|diag| {
                let NumberOrString::String(code) = diag.code.expect("Diagnostics didnt have code")
                else {
                    unreachable!("Diagnostic didnt have string code")
                };

                code
            })
            .collect()
    }

    #[test]
    fn property_overshadowed() {
        let result = run_diagnostics(
            "
        button {
            color: red;
            color: green;
        }
        ",
        );

        assert_eq!(result, vec!["css_overshadowed", "css_overshadowed_hint"]);
    }

    #[test]
    fn property_overshadowed_3() {
        let result = run_diagnostics(
            "
        button {
            color: red;
            color: green;
            color: blue;
        }
        ",
        );

        assert_eq!(
            result,
            vec![
                "css_overshadowed",
                "css_overshadowed_hint",
                "css_overshadowed",
            ]
        );
    }

    #[test]
    fn invalid_units() {
        let result = run_diagnostics(
            "
        button {
            font-size: 10pacmen;
        }
        ",
        );

        assert_eq!(result, vec!["css_invalid_unit"]);
    }

    #[test]
    fn uneeded_units() {
        let result = run_diagnostics(
            "
        button {
            font-size: 0px;
        }
        ",
        );

        assert_eq!(result, vec!["css_zero_unit"]);
    }

    #[test]
    fn zero_needed_units() {
        let result = run_diagnostics(
            "
        button {
            transition: 0s;
        }
        ",
        );

        assert!(result.is_empty());
    }

    #[test]
    fn hex_len_5() {
        let result = run_diagnostics(
            "
        button {
            color: #12345;
        }
        ",
        );

        assert_eq!(result, vec!["css_invalid_hex"]);
    }

    #[test]
    fn hex_len_5_alphanumeric() {
        let result = run_diagnostics(
            "
        button {
            color: #abcde;
        }
        ",
        );

        assert_eq!(result, vec!["css_invalid_hex"]);
    }

    #[parameterized(
        bulma = {"bulma.css"},
        bootstrap = {"bootstrap.css"},
        foundation = {"foundation.css"},
        materialize = {"materialize.css"},
        fomantic = {"materialize.css"},
        vanilla = {"vanilla.css"},
        tachyon = {"tachyon.css"},
    )]
    fn css_framework_sanity_check(file: &str) {
        let content = std::fs::read_to_string(format!("../test_data/{file}")).unwrap();

        let parser = construct_css_parser().unwrap();
        let document = ripple_parser::Document::parse(content, parser).unwrap();
        let result = Css::default().diagnostics(&document);

        for lint in result {
            if lint.severity == Some(DiagnosticSeverity::ERROR) {
                // We are not in control of syntax lints
                // (well not in control as we just report what TS tells us)
                // ((And theres a good few parsing bugs in the grammar unfourtnatly))
                // https://github.com/tree-sitter/tree-sitter-css/issues/67
                // https://github.com/tree-sitter/tree-sitter-css/issues/68
                if lint.code == Some(NumberOrString::String("css_invalid_syntax".to_string())) {
                    continue;
                }

                dbg!(lint);
                panic!("Error lint triggered on know good css file.");
            }
        }
    }
}

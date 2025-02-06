//! Advanced rust based html and css lsp.
use std::path::PathBuf;
use std::time::Instant;

use clap::{Parser, Subcommand};
use surf::lsp::Language;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Default, Debug)]
enum Command {
    #[default]
    Lsp,
    Lint {
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or_default();

    match command {
        Command::Lsp => {
            surf::lsp::lsp_main();
        }
        Command::Lint { path } => {
            lint_tree(path);
        }
    }
}

fn lint_tree(path: PathBuf) {
    if path.is_file() {
        if path.extension().unwrap_or_default() == "css" {
            lint(path);
        }
    } else {
        for file in std::fs::read_dir(path).unwrap().flatten() {
            let file = file.path();
            lint_tree(file);
        }
    }
}

fn lint(path: PathBuf) {
    let content = std::fs::read_to_string(&path).unwrap();

    let parser = ripple_parser::construct_css_parser().unwrap();
    let document = ripple_parser::Document::parse(content, parser).unwrap();

    let start = Instant::now();
    let diags = surf::css::Css::default().diagnostics(&document);
    let took = start.elapsed();

    println!(
        "🔥 Found {} problems in {} in {} ms",
        diags.len(),
        path.file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default(),
        took.as_millis()
    );
}

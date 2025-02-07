use criterion::{Criterion, criterion_group, criterion_main};
use surf::lsp::Language;

fn diagnostics(bencher: &mut Criterion) {
    let css = surf::css::Css::default();

    for file in [
        "bulma",
        //"bootstrap",
        //"foundation",
        //"materialize",
        "fomantic",
        //"vanilla",
        //"tachyon",
    ] {
        let content = std::fs::read_to_string(format!("../test_data/{file}.css")).unwrap();
        let parser = ripple_parser::construct_css_parser().unwrap();
        let document = ripple_parser::Document::parse(content, parser).unwrap();

        bencher.bench_function(file, |b| b.iter(|| css.diagnostics(&document)));
    }
}

criterion_group!(benches, diagnostics);
criterion_main!(benches);

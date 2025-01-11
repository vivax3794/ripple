VERSION 0.8
IMPORT github.com/earthly/lib/rust AS rust

env:
    FROM rustlang/rust:nightly-slim
    WORKDIR /app

    ENV CARGO_TERM_COLOR=always
    DO rust+INIT --keep_fingerprints=true

COPY_SOURCE:
    FUNCTION
    COPY --keep-ts . ./

tests-unit:
    FROM +env
    RUN rustup component add llvm-tools-preview
    DO rust+CARGO --args="install --locked cargo-nextest"
    DO rust+CARGO --args="install --locked cargo-llvm-cov"
    DO +COPY_SOURCE

    DO rust+CARGO --args="llvm-cov --html --output-dir output nextest --workspace --all-features --no-fail-fast"
    SAVE ARTIFACT ./output/html report AS LOCAL ./artifacts/coverage

tests-mutation:
    FROM +env
    DO rust+CARGO --args="install --locked cargo-mutants"
    DO +COPY_SOURCE

    DO rust+CARGO --args="mutants --jobs 4 -- --all-features"

tests:
    BUILD +tests-unit
    BUILD +tests-mutation


lint-clippy:
    FROM +env
    RUN rustup component add clippy
    DO +COPY_SOURCE
    
    DO rust+CARGO --args="clippy --all --workspace --all-features -- -D warnings"

lint-fmt:
    FROM +env
    RUN rustup component add rustfmt
    DO +COPY_SOURCE
    DO rust+CARGO --args="fmt --all -- --check"

lint:
    BUILD +lint-clippy
    BUILD +lint-fmt

ci:
    BUILD +lint
    BUILD +tests

benchmarks-unit:
    FROM +env
    DO +COPY_SOURCE
    DO rust+CARGO --args="bench --bench bench"

benchmarks:
    BUILD +benchmarks-unit

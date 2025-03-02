[working-directory: './natrix']
test:
    rustup run stable wasm-pack test --headless --chrome 
    rustup run nightly wasm-pack test --headless --chrome --features nightly

[working-directory: './natrix']
test_full: test
    rustup run stable wasm-pack test --headless --firefox
    rustup run nightly wasm-pack test --headless --firefox --features nightly

lint:
    cargo fmt --all
    cargo +stable clippy
    cargo +nightly clippy --all-features

[working-directory: './docs']
book:
    mdbook serve --open

docs:
    cargo doc --open -p natrix --lib

publish: test_full lint
    cargo publish -p natrix_macros
    cargo publish -p natrix

clean:
    cargo clean
    rm -rv docs/book || true

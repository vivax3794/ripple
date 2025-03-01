[working-directory: './ripple']
test:
    rustup run stable wasm-pack test --headless --chrome 
    rustup run nightly wasm-pack test --headless --chrome --features nightly

[working-directory: './ripple']
test_full:
    rustup run stable wasm-pack test --headless --chrome --firefox
    rustup run nightly wasm-pack test --headless --chrome --firefox --features nightly

lint:
    cargo fmt --all
    cargo +stable clippy
    cargo +nightly clippy --all-features

[working-directory: './docs']
book:
    mdbook serve --open

[working-directory: './test_project']
dev:
    trunk serve --port 8000 --watch ..

[working-directory: './test_project']
dev_release:
    trunk serve --port 8000 --watch .. --release

[working-directory: './test_project']
build:
    trunk build --release
    cd "./dist" && wasm-snip --snip-rust-panicking-code --snip-rust-fmt-code -o test_project_bg.wasm test_project_bg.wasm
    cd "./dist" && wasm-opt --strip-debug --strip-dwarf --strip-producers --disable-exception-handling -Oz -o test_project_bg.wasm test_project_bg.wasm --enable-bulk-memory-opt

[working-directory: './test_project/dist']
serve_build: build
    python -m http.server

test: test_native test_web

test_native:
    rustup run stable cargo nextest run --lib
    rustup run nightly cargo nextest run --features nightly --lib
    rustup run stable cargo nextest run --features force_unsafe_optimization --lib
    rustup run nightly cargo nextest run --features nightly,force_unsafe_optimization --lib

[working-directory: './ripple']
test_web:
    rustup run stable wasm-pack test --headless --chrome
    rustup run nightly wasm-pack test --headless --chrome --features nightly
    rustup run stable wasm-pack test --headless --chrome --features force_unsafe_optimization 
    rustup run nightly wasm-pack test --headless --chrome --features nightly,force_unsafe_optimization

lint:
    cargo clippy
    cargo clippy --all-features --release

[working-directory: './test_project']
dev:
    trunk serve --port 8000 --watch ..

[working-directory: './test_project']
dev_release:
    trunk serve --port 8000 --watch .. --release

[working-directory: './test_project']
build:
    trunk build --release

[working-directory: './test_project/dist']
serve_build: build
    python -m http.server


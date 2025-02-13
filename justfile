test:
    RUSTFLAGS=-Awarnings cargo nextest run --no-tests warn
    RUSTFLAGS=-Awarnings wasm-pack test --headless --chrome --firefox ripple

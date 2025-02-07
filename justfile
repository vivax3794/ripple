nvim: surf
    nvim scratch/test.css

helix: surf
    helix scratch/test.css

test:
    cargo nextest run

bench: bench_surf

bench_surf: bench_surf_cli
    cargo bench --bench frameworks_diagnostics -- --noplot --measurement-time 15

bench_surf_cli: surf
    hyperfine --warmup 10 "./bin/surf lint test_data"

lint_data: surf
    ./bin/surf lint test_data

surf:
    cargo build --release -p surf
    mv ./target/release/surf bin/surf || true

flame_surf:
    RUSTFLAGS="-C force-frame-pointers=yes" CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p surf -- lint test_data

release: clean && release_surf

release_surf:
    cargo pgo instrument test -p surf
    cargo pgo optimize build -p surf
    mv ./target/x86_64-unknown-linux-gnu/release/surf bin/surf

clean:
    rm perf.data || true
    rm perf.data.old || true
    rm flamegraph.svg || true
    rm -r bin || true
    mkdir bin

# We are going to have more projects in this repo, hence splitting bench up

nvim: surf
    nvim scratch/test.css

helix: surf
    helix scratch/test.css

ci: test bench

test:
    cargo nextest run

bench: bench_surf

bench_surf:
    cargo criterion --bench frameworks_diagnostics --plotting-backend disabled

bench_surf_cli: surf
    hyperfine --warmup 10 "./target/release/surf lint test_data/bulma.css"
    hyperfine --warmup 10 "./target/release/surf lint test_data/bootstrap.css"
    hyperfine --warmup 10 "./target/release/surf lint test_data/foundation.css"
    hyperfine --warmup 10 "./target/release/surf lint test_data/materialize.css"

lint_data: surf
    ./target/release/surf lint test_data

surf:
    cargo build -p surf --release

profile_surf:
    RUSTFLAGS="-C force-frame-pointers=yes" CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p surf -- lint test_data/bulma.css

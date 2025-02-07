nvim: surf
    nvim scratch/test.css

helix: surf
    helix scratch/test.css

ci: test bench

test:
    cargo nextest run

bench: bench_surf bench_surf_cli

bench_surf: profile
    cargo pgo optimize bench -- --bench frameworks_diagnostics -- --noplot --color always

bench_surf_cli: surf lint_data
    hyperfine --warmup 10 "./target/release/surf lint test_data"

lint_data: surf
    ./target/release/surf lint test_data

surf: profile
    cargo pgo optimize build -- -p surf

profile:
    cargo pgo instrument test

flame_surf:
    RUSTFLAGS="-C force-frame-pointers=yes" CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -p surf -- lint test_data

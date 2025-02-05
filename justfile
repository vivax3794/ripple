surf:
    cargo build -p surf

nvim: surf
    nvim scratch/test.css

helix: surf
    helix scratch/test.css

test:
    cargo nextest run

all: check format lint test

build:
    cargo build

format:
    cargo fmt

check:
    cargo check

lint:
    cargo clippy -- -D warnings

test:
    cargo nextest run

cover:
    # cargo install cargo-tarpaulin
    cargo tarpaulin --ignore-tests --out html

pin:
    ratchet pin .github/workflows/*.yml

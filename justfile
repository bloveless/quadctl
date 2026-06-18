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
    cargo test

test-cover:
    cargo tarpaulin --ignore-tests

pin:
    ratchet pin .github/workflows/*.yml

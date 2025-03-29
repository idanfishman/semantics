fmt:
    cargo fmt
lint:
    cargo clippy -- -D warnings -A dead_code
test:
    cargo test

precommit: fmt lint test
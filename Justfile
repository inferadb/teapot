# Default recipe: run all checks
default: check

# Run all checks (test, lint, format)
check: test lint fmt-check

# Run tests
test:
    cargo +1.92 test

# Run clippy linter
lint:
    cargo +1.92 clippy --all-targets -- -D warnings

# Check formatting
fmt-check:
    cargo +nightly fmt --check

# Format code
fmt:
    cargo +nightly fmt

# Check for unused dependencies
udeps:
    cargo +nightly udeps --all-targets

# Run an example (e.g., just example styling)
example name:
    cargo +1.92 run --example {{name}}

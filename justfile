# Parry Justfile

# Default recipe
default:
    @just --list

# Build all crates
build:
    cargo build --release

# Run tests
test:
    cargo test --all

# Run tests with output
test-verbose:
    cargo test --all -- --nocapture

# Check code (no build)
check:
    cargo check --all

# Format code
fmt:
    cargo fmt --all

# Lint code
clippy:
    cargo clippy --all -- -D warnings

# Run Parry on itself (bootstrap)
bootstrap:
    cargo run -- check --validators tailwind,imports crates/

# Install locally
install: build
    cargo install --path crates/cli

# Clean build artifacts
clean:
    cargo clean

# Watch for changes and run tests
watch:
    cargo watch -x test

# Release build
release: clean fmt clippy test
    cargo build --release

# Generate docs
docs:
    cargo doc --all --no-deps --open

# Run with debug output
debug *args:
    cargo run -- {{args}} --verbose

# Print version
version:
    cargo run -- --version

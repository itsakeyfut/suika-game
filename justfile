# =============================================================================
# justfile - Daily development workflow
# =============================================================================
#
# Suika Game - Bevy Game Development Workflow
#
# =============================================================================

# Update local main branch
new:
    git checkout main && git fetch && git pull origin main

# === Build Commands ===

# Build workspace in debug mode
build:
    cargo build --workspace

# Build game in debug mode (faster compile, more logging)
dev-build:
    cargo build -p suika-game

# Build game in release mode (optimized, minimal logging)
release-build:
    cargo build -p suika-game --release

# === Run Commands ===

# Run game (default)
run:
    cargo run -p suika-game

# Run game in debug mode with debug logging
dev:
    RUST_LOG=debug,wgpu=warn,wgpu_hal=warn,naga=warn cargo run -p suika-game

# Run game in release mode
release:
    cargo run -p suika-game --release

# === Code Quality ===

# Format code
fmt:
    cargo fmt --all

# Run clippy
clippy:
    cargo clippy --workspace -- -D warnings

# Quick check (format + clippy)
check:
    cargo fmt --all -- --check && cargo clippy --workspace -- -D warnings

# === Testing ===

# Run all tests (unit + integration) for all crates
test:
    cargo test --workspace

# Run unit tests: all crates / specific crate / specific test in crate
# Examples:
#   just unit-test                      # All unit tests
#   just unit-test suika-game-core      # All unit tests in suika-game-core
#   just unit-test suika-game-core test_fruit_merge  # Specific test
unit-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --lib
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --lib
    else
        cargo test -p {{crate}} --lib {{test}}
    fi

# Run integration tests: all crates / specific crate / specific test in crate
# Examples:
#   just integration-test                      # All integration tests
#   just integration-test suika-game-core      # All integration tests in suika-game-core
#   just integration-test suika-game-core test_collision  # Specific test
integration-test crate="" test="":
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "{{crate}}" ]; then
        cargo test --workspace --tests
    elif [ -z "{{test}}" ]; then
        cargo test -p {{crate}} --tests
    else
        cargo test -p {{crate}} --tests {{test}}
    fi

# Run tests sequentially (saves memory)
test-seq:
    cargo test --workspace -- --test-threads=1

# === WASM Build ===

# Build for WASM
wasm-build:
    cargo build --release --target wasm32-unknown-unknown -p suika-game
    wasm-bindgen --out-dir ./web --target web ./target/wasm32-unknown-unknown/release/suika-game.wasm

# Run WASM locally
wasm-serve: wasm-build
    basic-http-server web

# === Clean ===

# Clean build artifacts
clean:
    cargo clean

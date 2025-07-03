#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

echo "Building the svgn project..."
# Build the project in release mode for optimized binaries
cargo build --release

echo "Running tests..."
# Run all unit and integration tests
cargo test

echo "Running linter (clippy)..."
# Run clippy to catch common mistakes and improve code quality
cargo clippy -- -D warnings

echo "Checking code formatting..."
# Check if code is formatted according to rustfmt rules
cargo fmt --check

echo "Build and verification complete."
echo "To run the optimized binary, use: ./target/release/svgn"

#!/bin/bash
# Quick validation script to run before pushing

set -e

echo "🔍 Running pre-push validation..."

echo "✅ Checking formatting..."
cargo fmt --all -- --check

echo "✅ Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "✅ Running tests..."
cargo test --all-features

echo "✅ Building release..."
cargo build --release

echo "🎉 All checks passed! Safe to push to GitHub."
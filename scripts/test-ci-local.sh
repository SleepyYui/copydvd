#!/bin/bash
# Local CI testing script

echo "Testing CI steps locally..."

# Simulate Ubuntu CI environment
docker run --rm -v "$(pwd)":/workspace -w /workspace ubuntu:latest bash -c "
  apt-get update
  apt-get install -y build-essential pkg-config libx11-dev libxrandr-dev libxi-dev libgl1-mesa-dev libglib2.0-dev libglib2.0-dev-bin libgtk-3-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf2.0-dev libatk1.0-dev curl

  # Install Rust
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source ~/.cargo/env

  # Run CI steps
  echo '=== Running cargo fmt check ==='
  cargo fmt --all -- --check

  echo '=== Running clippy ==='
  cargo clippy --all-targets --all-features -- -D warnings

  echo '=== Running tests ==='
  cargo test --release --verbose --all-features

  echo '=== Building release ==='
  cargo build --release --target x86_64-unknown-linux-gnu --all-features
"
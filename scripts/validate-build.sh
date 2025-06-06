#!/bin/bash
set -euo pipefail

echo "🔍 Validating CopyDVD build pipeline..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

echo_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

echo_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Function to run a command and capture its result
run_check() {
    local description="$1"
    local command="$2"
    
    echo "🔄 $description..."
    if eval "$command" > /dev/null 2>&1; then
        echo_success "$description"
        return 0
    else
        echo_error "$description"
        return 1
    fi
}

# Check prerequisites
echo "📋 Checking prerequisites..."

# Check Rust installation
if ! command -v cargo &> /dev/null; then
    echo_error "Cargo not found. Please install Rust: https://rustup.rs/"
    exit 1
fi
echo_success "Rust/Cargo installed"

# Check formatting
run_check "Code formatting" "cargo fmt --all -- --check"

# Check clippy
run_check "Clippy lints" "cargo clippy --all-targets --all-features -- -D warnings"

# Build debug
run_check "Debug build" "cargo build --all-features"

# Run tests
run_check "Tests" "cargo test --all-features"

# Build release
run_check "Release build" "cargo build --release --all-features"

# Check if cross is available for ARM testing
if command -v cross &> /dev/null; then
    echo_success "Cross-compilation tool available"
    
    # Test ARM Linux build (CLI only)
    if run_check "ARM Linux build (CLI-only)" "cross build --target aarch64-unknown-linux-gnu --no-default-features"; then
        echo_success "ARM Linux cross-compilation working"
    else
        echo_warning "ARM Linux cross-compilation failed (this is expected in some environments)"
    fi
else
    echo_warning "Cross not installed - ARM Linux builds will be done in CI"
fi

# Verify binary exists
if [[ -f "target/release/copydvd" ]] || [[ -f "target/release/copydvd.exe" ]]; then
    echo_success "Release binary created successfully"
    
    # Get binary info
    if [[ -f "target/release/copydvd" ]]; then
        ls -lh target/release/copydvd
        file target/release/copydvd || true
    else
        ls -lh target/release/copydvd.exe
        file target/release/copydvd.exe || true
    fi
else
    echo_error "Release binary not found"
    exit 1
fi

echo ""
echo_success "🎉 All validation checks passed!"
echo "📦 Ready for CI/CD pipeline"
echo ""
echo "💡 To test the full pipeline:"
echo "   1. Commit your changes"
echo "   2. Push to v2 branch" 
echo "   3. Create a tag: git tag v1.0.0 && git push origin v1.0.0"
echo "   4. Or trigger manually via GitHub Actions web interface"
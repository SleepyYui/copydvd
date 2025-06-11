#!/bin/bash

# Local development script with Docker caching for fast iterations
# Usage: ./scripts/local-dev.sh [command]
# Commands: quality, build, test, shell

set -e

COMMAND=${1:-quality}
PROJECT_ROOT=$(cd "$(dirname "$0")/.." && pwd)
DOCKER_IMAGE="copydvd-dev:latest"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🐳 CopyDVD Local Development Environment${NC}"
echo -e "${BLUE}Command: $COMMAND${NC}"

# Check if Docker image exists, build if not
if ! docker image inspect $DOCKER_IMAGE > /dev/null 2>&1; then
    echo -e "${YELLOW}📦 Building development Docker image (one-time setup)...${NC}"
    
    cat > "$PROJECT_ROOT/Dockerfile.dev" << 'EOF'
FROM rust:1.75-bullseye

# Install system dependencies (cached layer)
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libx11-dev \
    libxrandr-dev \
    libxi-dev \
    libgl1-mesa-dev \
    libglib2.0-dev \
    libglib2.0-dev-bin \
    libgtk-3-dev \
    libcairo2-dev \
    libpango1.0-dev \
    libgdk-pixbuf2.0-dev \
    libatk1.0-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust components (cached layer)
RUN rustup component add rustfmt clippy

# Set working directory
WORKDIR /app

# Pre-create cargo registry cache
RUN cargo search --limit 0 || true

# Default command
CMD ["bash"]
EOF

    docker build -f Dockerfile.dev -t $DOCKER_IMAGE "$PROJECT_ROOT"
    echo -e "${GREEN}✅ Development image built successfully${NC}"
fi

# Docker run function with consistent options
run_docker() {
    docker run --rm \
        -v "$PROJECT_ROOT:/app" \
        -v cargo-cache:/usr/local/cargo/registry \
        -v target-cache:/app/target \
        -w /app \
        $DOCKER_IMAGE \
        "$@"
}

case $COMMAND in
    "quality"|"q")
        echo -e "${BLUE}🔍 Running quality checks in Docker...${NC}"
        echo -e "${YELLOW}⚡ Using cached dependencies and toolchain${NC}"
        
        run_docker bash -c "
            echo '📝 Checking formatting...'
            cargo fmt --all -- --check
            
            echo '🔍 Running clippy...'
            cargo clippy --all-targets --all-features -- -D warnings
            
            echo '🧪 Running tests...'
            cargo test --all-features
            
            echo '✅ All quality checks passed!'
        "
        ;;
        
    "build"|"b")
        echo -e "${BLUE}🔨 Building in Docker...${NC}"
        TARGET=${2:-"x86_64-unknown-linux-gnu"}
        
        run_docker bash -c "
            echo '🎯 Building for target: $TARGET'
            cargo build --release --target $TARGET --all-features
            echo '✅ Build completed!'
            ls -la target/$TARGET/release/
        "
        ;;
        
    "test"|"t")
        echo -e "${BLUE}🧪 Running tests in Docker...${NC}"
        TEST_PATTERN=${2:-""}
        
        if [[ -n "$TEST_PATTERN" ]]; then
            run_docker cargo test --all-features -- "$TEST_PATTERN"
        else
            run_docker cargo test --all-features
        fi
        ;;
        
    "shell"|"sh")
        echo -e "${BLUE}🐚 Starting development shell...${NC}"
        echo -e "${YELLOW}💡 Use 'cargo build', 'cargo test', etc. inside the container${NC}"
        
        run_docker bash
        ;;
        
    "clean")
        echo -e "${BLUE}🧹 Cleaning Docker caches...${NC}"
        docker volume rm cargo-cache target-cache 2>/dev/null || true
        docker rmi $DOCKER_IMAGE 2>/dev/null || true
        rm -f "$PROJECT_ROOT/Dockerfile.dev"
        echo -e "${GREEN}✅ Caches cleaned${NC}"
        ;;
        
    "help"|"h"|*)
        echo -e "${YELLOW}Usage: $0 [command]${NC}"
        echo ""
        echo -e "${BLUE}Commands:${NC}"
        echo -e "  ${GREEN}quality, q${NC}     Run formatting, clippy, and tests"
        echo -e "  ${GREEN}build, b${NC}       Build release binary [target]"
        echo -e "  ${GREEN}test, t${NC}        Run tests [pattern]"
        echo -e "  ${GREEN}shell, sh${NC}      Open development shell"
        echo -e "  ${GREEN}clean${NC}          Clean Docker caches and image"
        echo -e "  ${GREEN}help, h${NC}        Show this help"
        echo ""
        echo -e "${BLUE}Examples:${NC}"
        echo -e "  $0 quality                    # Full quality check"
        echo -e "  $0 build                      # Build Linux binary"
        echo -e "  $0 test handbrake             # Run tests matching 'handbrake'"
        echo -e "  $0 shell                      # Interactive development"
        ;;
esac

echo -e "${GREEN}🎉 Done!${NC}"
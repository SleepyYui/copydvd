#!/bin/bash

# Setup script for automatic version bumping git hooks
# This script installs and configures the pre-commit hook for automatic version bumping

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔧 Setting up automatic version bumping git hooks...${NC}"

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo -e "${RED}❌ Error: Not in a git repository${NC}"
    echo -e "${YELLOW}💡 Run this script from the root of your git repository${NC}"
    exit 1
fi

# Check if Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ Error: Cargo.toml not found${NC}"
    echo -e "${YELLOW}💡 This script should be run from a Rust project root${NC}"
    exit 1
fi

# Create .git/hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy pre-commit hook
if [ -f ".githooks/pre-commit" ]; then
    cp .githooks/pre-commit .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo -e "${GREEN}✅ Pre-commit hook installed${NC}"
else
    echo -e "${RED}❌ Error: .githooks/pre-commit not found${NC}"
    exit 1
fi

# Copy commit-msg hook
if [ -f ".githooks/commit-msg" ]; then
    cp .githooks/commit-msg .git/hooks/commit-msg
    chmod +x .git/hooks/commit-msg
    echo -e "${GREEN}✅ Commit-msg hook installed${NC}"
else
    echo -e "${RED}❌ Error: .githooks/commit-msg not found${NC}"
    exit 1
fi

# Copy post-commit hook
if [ -f ".githooks/post-commit" ]; then
    cp .githooks/post-commit .git/hooks/post-commit
    chmod +x .git/hooks/post-commit
    echo -e "${GREEN}✅ Post-commit hook installed${NC}"
else
    echo -e "${RED}❌ Error: .githooks/post-commit not found${NC}"
    exit 1
fi

# Configure git to use our hooks directory (optional, for team sharing)
git config core.hooksPath .githooks
echo -e "${GREEN}✅ Git configured to use .githooks directory${NC}"

# Make hooks executable
chmod +x .githooks/pre-commit .githooks/commit-msg .githooks/post-commit
echo -e "${GREEN}✅ Hooks made executable${NC}"

# Test the current version detection
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${BLUE}📦 Current project version: ${NC}$CURRENT_VERSION"

echo ""
echo -e "${GREEN}🎉 Setup complete!${NC}"
echo ""
echo -e "${BLUE}📋 How it works:${NC}"
echo -e "   • ${GREEN}Pre-commit hook:${NC} Automatically bumps version in Cargo.toml based on commit message"
echo -e "   • ${GREEN}Commit-msg hook:${NC} Validates commit messages"
echo -e "   • ${GREEN}Post-commit hook:${NC} Creates git tags after successful commits"
echo ""
echo -e "${BLUE}📋 Version bump rules:${NC}"
echo -e "   • ${GREEN}feat:${NC} commits will bump the ${YELLOW}minor${NC} version (0.1.0 → 0.2.0)"
echo -e "   • ${GREEN}fix:${NC} commits will bump the ${YELLOW}patch${NC} version (0.1.0 → 0.1.1)"
echo -e "   • ${GREEN}BREAKING CHANGE${NC} or ${GREEN}feat!${NC} will bump the ${YELLOW}major${NC} version (0.1.0 → 1.0.0)"
echo -e "   • Other commits (docs, chore, etc.) will bump the ${YELLOW}patch${NC} version"
echo ""
echo -e "${BLUE}📝 Example commit messages:${NC}"
echo -e "   • ${GREEN}feat: add new DVD scanning feature${NC} → minor bump"
echo -e "   • ${GREEN}fix: resolve HandBrake detection issue${NC} → patch bump"
echo -e "   • ${GREEN}feat!: redesign configuration system${NC} → major bump"
echo -e "   • ${GREEN}docs: update installation instructions${NC} → patch bump"
echo ""
echo -e "${YELLOW}💡 The version in Cargo.toml will be automatically updated and tagged on each commit!${NC}"
echo -e "${BLUE}ℹ️  Use ${GREEN}[skip version]${NC} in commit message to skip automatic version bumping${NC}"
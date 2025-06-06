# GitHub Actions Workflows

This directory contains the CI/CD workflows for the CopyDVD project.

## Workflows Overview

### 🚀 Main CI/CD Pipeline (`ci.yml`)

**Triggers:**
- Push to `v2` or `main` branches
- Pull requests to `v2` or `main` branches  
- Git tags starting with `v*`
- Manual dispatch with optional force release

**Jobs:**
1. **Quality** - Code formatting, linting, testing
2. **Build** - Cross-platform binary compilation
3. **Release** - Creates GitHub releases (tags only)

**Supported Platforms:**
- Linux: x86_64 (full), aarch64 (CLI-only)
- macOS: x86_64, aarch64 (both full-featured)
- Windows: x86_64 (full-featured)

### 🔄 Dependency Updates (`dependencies.yml`)

**Triggers:**
- Weekly schedule (Mondays at midnight UTC)
- Manual dispatch

**Purpose:**
- Automatically updates Cargo dependencies
- Validates all builds and tests still pass
- Creates pull requests for review

## Usage

### Creating a Release

#### Option 1: Git Tag (Recommended)
```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

#### Option 2: Manual Trigger
1. Go to GitHub Actions tab
2. Select "CI/CD Pipeline"
3. Click "Run workflow"
4. Check "Force create release"

### Development Workflow

1. **Push to v2**: Triggers quality checks and builds
2. **Create PR**: Runs full validation pipeline
3. **Tag release**: Triggers release creation with binaries

### Local Validation

Before pushing, run the validation script:
```bash
./scripts/validate-build.sh
```

## Build Matrix

| Platform | Target | Features | Status |
|----------|--------|----------|--------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` | Full (GUI + CLI) | ✅ |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | CLI only | ✅ |
| macOS Intel | `x86_64-apple-darwin` | Full (GUI + CLI) | ✅ |
| macOS Apple Silicon | `aarch64-apple-darwin` | Full (GUI + CLI) | ✅ |
| Windows x86_64 | `x86_64-pc-windows-msvc` | Full (GUI + CLI) | ✅ |

**Note**: ARM Linux is CLI-only due to GUI cross-compilation complexity.

## Troubleshooting

### Build Failures

1. Check the **Quality** job first - formatting/linting issues
2. Look at platform-specific **Build** jobs for compilation errors
3. Review **Release** job for artifact/upload issues

### Common Issues

- **Formatting**: Run `cargo fmt --all`
- **Clippy**: Run `cargo clippy --all-targets --all-features -- -D warnings`
- **ARM Linux**: Expected to be CLI-only, GUI builds will fail

### Workflow Debugging

Enable debug logging by adding this to your fork's secrets:
```
ACTIONS_STEP_DEBUG = true
ACTIONS_RUNNER_DEBUG = true
```

## Security

- Uses `GITHUB_TOKEN` for releases (no additional secrets needed)
- Artifacts are retained for 30 days
- Release assets are public (as expected for open source)

## Maintenance

- Workflows use pinned action versions for stability
- Dependencies are auto-updated weekly via dedicated workflow
- Rust cache is used to speed up builds
- Cross-compilation uses official cross-rs Docker images
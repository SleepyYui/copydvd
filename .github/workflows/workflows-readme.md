# GitHub Actions Workflows

This project uses multiple focused workflows that run in parallel for better performance and maintainability.

## Branch Strategy

- **`v2-dev`**: Development branch - **Quality checks only**
- **`v2`**: Production branch - **Full pipeline** (quality → build → installers → release)
- **`main`**: Legacy branch - **Quality checks only**

## Workflow Overview

### 1. `quality.yml` - Code Quality Checks
**Triggers**: Pushes/PRs to `v2`, `v2-dev`, `main`
- ✅ Code formatting (`cargo fmt`)
- ✅ Linting (`cargo clippy`) 
- ✅ Tests (`cargo test`)
- **Runtime**: ~3-5 minutes

### 2. `version.yml` - Version Management  
**Triggers**: Pushes to `v2` branch, tags, manual dispatch
- 🏷️ Auto-generates semantic versions
- 📝 Updates `Cargo.toml`
- 🏷️ Creates git tags
- **Runtime**: ~1 minute

### 3. `build.yml` - Binary Compilation
**Triggers**: After quality checks pass, pushes to `v2` **only**
- 🔨 Builds for all platforms in parallel:
  - Linux x86_64
  - macOS Intel & Apple Silicon  
  - Windows x86_64
- 📦 Uploads binary artifacts
- **Runtime**: ~8-12 minutes (parallel)

### 4. `installers.yml` - Package Creation
**Triggers**: After binaries are built (`v2` **only**)
- 📦 Creates macOS PKG installers (both architectures)
- 📦 Creates Windows MSI installer
- 🔧 Handles platform-specific packaging
- **Runtime**: ~5-8 minutes (parallel)

### 5. `release.yml` - GitHub Release
**Triggers**: After installers are created (`v2` **only**)
- 🤖 Generates AI-powered release notes
- 📋 Collects all artifacts
- 🚀 Creates GitHub release with assets
- **Runtime**: ~2-3 minutes

## Parallel Execution Flow

```
Push to v2 branch
      ↓
┌─────────────┬─────────────┐
│  quality    │  version    │  ← Run in parallel
│  (3-5 min)  │  (1 min)    │
└─────────────┴─────────────┘
      ↓
┌─────────────────────────────┐
│          build              │  ← 4 targets in parallel
│        (8-12 min)           │
└─────────────────────────────┘
      ↓
┌─────────────┬─────────────┐
│  macos-pkg  │ windows-msi │  ← Run in parallel  
│  (5-8 min)  │  (5-8 min)  │
└─────────────┴─────────────┘
      ↓
┌─────────────────────────────┐
│         release             │
│        (2-3 min)            │
└─────────────────────────────┘
```

**Total Time**: ~15-20 minutes (vs ~25-30 minutes sequential)

## Key Improvements

✅ **Parallel Execution**: Multiple jobs run simultaneously  
✅ **Faster Feedback**: Quality checks finish in 3-5 minutes  
✅ **Better Isolation**: Each workflow has a single responsibility  
✅ **Easier Debugging**: Focused logs per workflow  
✅ **Maintainable**: Smaller, focused files  
✅ **Artifact Management**: Clean separation of build outputs  

## Monitoring

- **Quality Status**: Shows immediately on PR/push
- **Build Progress**: Individual status per platform  
- **Release Status**: Clear pipeline progression
- **Artifact Downloads**: Available per workflow completion
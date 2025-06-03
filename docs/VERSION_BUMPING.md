# Version Bumping System

This project uses a hybrid version bumping system that provides automatic local version management with manual control over GitHub workflows.

## 🚀 Quick Setup

1. **Install the git hooks:**
   ```bash
   chmod +x setup-hooks.sh
   ./setup-hooks.sh
   ```

2. **Start using conventional commit messages:**
   ```bash
   git commit -m "feat: add new DVD scanning feature"
   # This will automatically bump the minor version (0.1.0 → 0.2.0) and create a tag
   ```

## 📋 How It Works

### 🔄 Local Git Hook (Pre-commit) - AUTOMATIC
- **Runs automatically** before each commit
- Analyzes your commit message using conventional commit patterns
- Updates `Cargo.toml` with the new version
- Creates a git tag (e.g., `v1.2.3`)
- Stages the updated files automatically
- **Always active** unless explicitly skipped with `[skip version]`

### ⚙️ GitHub Workflow - MANUAL ONLY
- **Only runs when manually triggered** from GitHub Actions tab
- Provides a safety net for when local hooks aren't used
- Allows manual version bumping with custom bump types
- **Never runs automatically** on pushes

### 🚀 Release Workflow - TAG-TRIGGERED
- **Only builds releases when version tags are created**
- Triggered automatically when pre-commit hook creates tags
- Can also be triggered manually for testing
- **No releases without version changes**

## 🎯 Commit Message Conventions

The pre-commit hook follows [Conventional Commits](https://www.conventionalcommits.org/) specification:

### Major Version Bump (1.0.0 → 2.0.0)
```bash
feat!: redesign the entire configuration system
# or
feat: add new feature
BREAKING CHANGE: removes support for old config format
```

### Minor Version Bump (1.0.0 → 1.1.0)
```bash
feat: add HandBrake auto-detection
feat(gui): implement new progress indicator
feat(cli): add batch processing support
```

### Patch Version Bump (1.0.0 → 1.0.1)
```bash
fix: resolve DVD detection on Windows
fix(build): correct cross-compilation issues
docs: update installation instructions
style: format code with rustfmt
refactor: improve error handling
perf: optimize scanning performance
test: add unit tests for DVD detection
chore: update dependencies
```

## 🛠️ Configuration

### Skipping Automatic Version Bumps
Add `[skip version]` or `[version skip]` to your commit message:
```bash
git commit -m "docs: update README [skip version]"
```

### Manual GitHub Version Bumping
1. Go to the GitHub Actions tab
2. Select "Manual Version Bump"
3. Click "Run workflow"
4. Choose the bump type:
   - **auto**: Analyze latest commit message
   - **major**: Force major version bump
   - **minor**: Force minor version bump
   - **patch**: Force patch version bump

### Manual Version Override
You can manually edit `Cargo.toml` if needed. The system will detect the manual change and continue from that version.

## 📁 File Structure

```
copydvd/
├── .githooks/
│   └── pre-commit          # Local git hook (automatic)
├── .github/workflows/
│   ├── version-bump.yml    # Manual version bumping
│   └── release.yml         # Tag-triggered releases
├── setup-hooks.sh          # Installation script
└── docs/
    └── VERSION_BUMPING.md  # This documentation
```

## 🔧 Workflow Details

### Pre-commit Hook Features
- **Automatic execution**: Runs on every commit
- **Git tag creation**: Creates `v1.2.3` tags automatically
- **Smart analysis**: Recognizes conventional commit patterns
- **Skip mechanism**: Respects `[skip version]` flags
- **Staging integration**: Automatically stages version changes

### GitHub Action Features
- **Manual control**: Only runs when explicitly triggered
- **Flexible bump types**: Choose specific version increments
- **Safety net**: Handles cases where local hooks weren't used
- **Team coordination**: Allows manual version coordination

### Release Workflow Features
- **Tag-triggered**: Only builds when versions change
- **Multi-platform**: Builds for all supported platforms
- **Smart releases**: No unnecessary releases without version changes
- **Manual override**: Can be triggered manually for testing

## 🎨 Output Examples

### Local Hook Output
```
🔍 Analyzing commit message: "feat: add new DVD scanning feature"
🔧 Automatic version bump enabled
📦 Current version: 0.1.0
🎯 Bump type: minor
🆕 New version: 0.2.0
📝 Updating Cargo.toml...
✅ Version bumped from 0.1.0 to 0.2.0
📁 Cargo.toml updated and staged
🏷️ Creating git tag v0.2.0...
📌 Git tag v0.2.0 created
🎉 Automatic version bump complete!
✨ MINOR version bump - New features detected
```

### GitHub Action Output
```
🎉 Manual version bump complete!
📦 Version: 0.2.0 → 0.3.0
🏷️ Tag: v0.3.0
🔄 Bump type: minor
```

## 🔍 Troubleshooting

### Pre-commit Hook Not Running
```bash
# Check if hooks are executable
ls -la .git/hooks/pre-commit

# Reinstall hooks
./setup-hooks.sh

# Verify hook is working
git commit -m "test: verify hook [skip version]"
```

### Wrong Version Bump
- Check your commit message format
- Use conventional commit prefixes (`feat:`, `fix:`, etc.)
- Remember that unknown patterns default to patch bumps
- Use `[skip version]` to prevent unwanted bumps

### GitHub Workflow Issues
- **Manual workflow not visible**: Check repository permissions
- **Manual workflow fails**: Verify write permissions to repository
- **Tags not pushing**: Ensure workflow has permission to create tags

### Release Not Building
- Check that a version tag was created (`git tag -l`)
- Verify release workflow has proper permissions
- Look for tag format issues (should be `v1.2.3`)

### Manual Reset
If you need to reset or fix versions:
```bash
# Edit Cargo.toml manually to set desired version
vim Cargo.toml

# Remove problematic tag if needed
git tag -d v1.2.3
git push origin :refs/tags/v1.2.3

# Commit with skip flag
git commit -am "chore: reset version to 1.0.0 [skip version]"
```

## 📊 Version History & Tags

The system maintains a clear version history through:
- **Git commits**: Descriptive messages with version changes
- **Git tags**: Automatic `v1.2.3` format tags
- **GitHub releases**: Triggered only when versions change
- **Semantic versioning**: Reflects the nature of changes

## 🤝 Team Collaboration

### For Team Projects:
1. **All team members should run** `./setup-hooks.sh`
2. **Use consistent commit message conventions**
3. **Local hooks handle most version bumping automatically**
4. **Use manual GitHub workflow for coordination when needed**
5. **Releases happen automatically when versions change**

### Workflow for Teams:
```bash
# Normal development (automatic version bumping)
git commit -m "feat: add new feature"  # Auto-bumps version + creates tag

# Skip version bumping when needed
git commit -m "docs: update docs [skip version]"  # No version change

# Manual coordination (when local hooks weren't used)
# 1. Go to GitHub Actions → "Manual Version Bump" → Run workflow
```

## 📝 Best Practices

1. **Use descriptive commit messages**: They become part of your version history
2. **Embrace automatic bumping**: Let the pre-commit hook handle versions
3. **Use skip flags judiciously**: Only skip when changes don't warrant version bumps
4. **Review version changes**: Check that the bump type matches your intent
5. **Use breaking change notation**: Be explicit about API changes with `!` or `BREAKING CHANGE`
6. **Coordinate with team**: Use manual GitHub workflow when coordination is needed
7. **Trust the automation**: The system ensures consistent versioning

## 🎯 When Each System Runs

| Scenario | Pre-commit Hook | GitHub Workflow | Release |
|----------|----------------|-----------------|---------|
| Normal commit | ✅ Auto-runs | ❌ Never | ✅ If tag created |
| Commit with `[skip version]` | ❌ Skipped | ❌ Never | ❌ No tag |
| Manual GitHub trigger | ❌ Not involved | ✅ Manual | ✅ If tag created |
| Push without local hooks | ❌ Already committed | ⚙️ Use manual | ❌ Until version bumped |

## 🌟 Benefits of This System

- **🔄 Automatic local development**: No manual version management needed
- **⚙️ Manual GitHub control**: Full control over remote version operations  
- **🚀 Smart releases**: Only build when there's something new to release
- **🤝 Team-friendly**: Works whether team uses hooks consistently or not
- **📊 Clean history**: Every version change is intentional and documented

This hybrid system gives you the best of both worlds: automatic local productivity with manual control over important remote operations! 🎉
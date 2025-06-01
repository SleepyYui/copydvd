# Automatic Version Bumping System

This project includes an automatic version bumping system that increments the version in `Cargo.toml` based on conventional commit messages.

## 🚀 Quick Setup

1. **Install the git hooks:**
   ```bash
   chmod +x setup-hooks.sh
   ./setup-hooks.sh
   ```

2. **Start using conventional commit messages:**
   ```bash
   git commit -m "feat: add new DVD scanning feature"
   # This will automatically bump the minor version (0.1.0 → 0.2.0)
   ```

## 📋 How It Works

### Local Git Hook (Pre-commit)
- Runs before each commit
- Analyzes your commit message
- Updates `Cargo.toml` with the new version
- Stages the updated files automatically

### GitHub Action (On Push)
- Runs when you push to `main`, `master`, or `v2` branches
- Creates version bump commits and git tags
- Handles cases where local hooks weren't used

## 🎯 Commit Message Conventions

The system follows [Conventional Commits](https://www.conventionalcommits.org/) specification:

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

### Skipping Version Bumps
Add `[skip version]` or `[version skip]` to your commit message:
```bash
git commit -m "docs: update README [skip version]"
```

### Manual Version Override
You can manually edit `Cargo.toml` if needed. The system will detect the manual change and continue from that version.

## 📁 File Structure

```
copydvd/
├── .githooks/
│   └── pre-commit          # Local git hook script
├── .github/workflows/
│   └── version-bump.yml    # GitHub Action for remote bumping
├── setup-hooks.sh          # Installation script
└── docs/
    └── VERSION_BUMPING.md  # This documentation
```

## 🔧 Advanced Usage

### Custom Bump Types
The system recognizes these patterns for patch bumps:
- `bug`, `hotfix`, `patch`, `update`, `improve`, `enhance`, `optimize`

### GitHub Action Features
- **Automatic tagging**: Creates git tags like `v1.2.3`
- **Smart detection**: Only runs on actual code changes
- **Conflict prevention**: Uses `[skip version]` to prevent infinite loops

## 🎨 Output Examples

### Local Hook Output
```
🔍 Analyzing commit message: "feat: add new DVD scanning feature"
📦 Current version: 0.1.0
🎯 Bump type: minor
🆕 New version: 0.2.0
📝 Updating Cargo.toml...
✅ Version bumped from 0.1.0 to 0.2.0
📁 Cargo.toml updated and staged
🎉 Automatic version bump complete!
✨ MINOR version bump - New features detected
```

### GitHub Action Output
```
🎉 Version bump complete!
📦 Version: 0.1.0 → 0.2.0
🏷️ Tag: v0.2.0
🔄 Bump type: minor
```

## 🔍 Troubleshooting

### Hook Not Running
```bash
# Check if hooks are executable
ls -la .git/hooks/pre-commit

# Reinstall hooks
./setup-hooks.sh
```

### Wrong Version Bump
- Check your commit message format
- Use conventional commit prefixes (`feat:`, `fix:`, etc.)
- Remember that unknown patterns default to patch bumps

### GitHub Action Not Triggering
- Ensure you're pushing to `main`, `master`, or `v2` branches
- Check that you didn't include `[skip version]` in the commit message
- Verify the action has write permissions to the repository

### Manual Reset
If you need to reset the version:
```bash
# Edit Cargo.toml manually to set desired version
vim Cargo.toml

# Commit with skip flag
git commit -am "chore: reset version to 1.0.0 [skip version]"
```

## 📊 Version History

The system maintains a clear version history through:
- Git commits with descriptive messages
- Git tags for each version
- Semantic versioning that reflects the nature of changes

## 🤝 Team Collaboration

For team projects:
1. All team members should run `./setup-hooks.sh`
2. Use consistent commit message conventions
3. The GitHub Action provides a safety net for missed local bumps
4. Version bumps are automatically included in release workflows

## 📝 Best Practices

1. **Use descriptive commit messages**: They become part of your version history
2. **Group related changes**: Make logical commits that deserve version bumps
3. **Test before committing**: The version bump happens automatically
4. **Review version changes**: Check that the bump type matches your intent
5. **Use breaking change notation**: Be explicit about API changes

This system ensures your project always has accurate, semantic versions that reflect the evolution of your codebase! 🎉
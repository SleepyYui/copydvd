#!/bin/bash
set -e

# Version bump script for DVD Ripper
# Usage: ./scripts/bump_version.sh [major|minor|patch]

if [ $# -eq 0 ]; then
    echo "Usage: $0 [major|minor|patch]"
    echo "Current version: $(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')"
    exit 1
fi

BUMP_TYPE=$1
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "Current version: $CURRENT_VERSION"

# Parse current version
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

# Bump version based on type
case $BUMP_TYPE in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
    *)
        echo "Invalid bump type. Use: major, minor, or patch"
        exit 1
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
echo "New version: $NEW_VERSION"

# Update Cargo.toml
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
cargo update -p copydvd

echo "Version updated from $CURRENT_VERSION to $NEW_VERSION"
echo ""
echo "Next steps:"
echo "1. Review and commit the changes:"
echo "   git add Cargo.toml Cargo.lock"
echo "   git commit -m \"chore: bump version to $NEW_VERSION\""
echo ""
echo "2. Push to v2 branch to trigger release:"
echo "   git push origin v2"
echo ""
echo "3. The CI will automatically create a release with tag v$NEW_VERSION"

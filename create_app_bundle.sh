#!/bin/bash

# Build script to create a proper macOS app bundle for Copy DVD

set -e

APP_NAME="Copy DVD"
BUNDLE_NAME="Copy DVD.app"
BUNDLE_ID="com.sleepyyui.copydvd"
VERSION="0.1.11"

echo "Building Copy DVD app bundle..."

# Build the release binary
echo "Building release binary..."
cargo build --release

# Create app bundle structure
echo "Creating app bundle structure..."
mkdir -p "$BUNDLE_NAME/Contents/MacOS"
mkdir -p "$BUNDLE_NAME/Contents/Resources"

# Copy the binary
echo "Copying binary..."
cp "target/release/copydvd" "$BUNDLE_NAME/Contents/MacOS/"

# Copy Info.plist
echo "Copying Info.plist..."
cp "Info.plist" "$BUNDLE_NAME/Contents/"

# Create and copy icon
echo "Creating app icon..."
if [ -f "resources/icons/icon-512.png" ]; then
    # Create icns file using iconutil (macOS built-in tool)
    mkdir -p "icon.iconset"
    
    # Copy all icon sizes to iconset
    cp "resources/icons/icon-16.png" "icon.iconset/icon_16x16.png"
    cp "resources/icons/icon-32.png" "icon.iconset/icon_16x16@2x.png"
    cp "resources/icons/icon-32.png" "icon.iconset/icon_32x32.png"
    cp "resources/icons/icon-64.png" "icon.iconset/icon_32x32@2x.png"
    cp "resources/icons/icon-128.png" "icon.iconset/icon_128x128.png"
    cp "resources/icons/icon-256.png" "icon.iconset/icon_128x128@2x.png"
    cp "resources/icons/icon-256.png" "icon.iconset/icon_256x256.png"
    cp "resources/icons/icon-512.png" "icon.iconset/icon_256x256@2x.png"
    cp "resources/icons/icon-512.png" "icon.iconset/icon_512x512.png"
    if [ -f "resources/icons/icon-1024.png" ]; then
        cp "resources/icons/icon-1024.png" "icon.iconset/icon_512x512@2x.png"
    fi
    
    # Create icns file
    iconutil -c icns "icon.iconset" -o "$BUNDLE_NAME/Contents/Resources/icon.icns"
    
    # Clean up
    rm -rf "icon.iconset"
    
    echo "Icon created successfully!"
else
    echo "Warning: Icon files not found, skipping icon creation"
fi

# Copy resources
echo "Copying resources..."
if [ -d "resources" ]; then
    cp -r "resources" "$BUNDLE_NAME/Contents/Resources/"
fi

# Set executable permissions
chmod +x "$BUNDLE_NAME/Contents/MacOS/copydvd"

echo "App bundle created successfully: $BUNDLE_NAME"
echo "You can now run the app by double-clicking it or using: open '$BUNDLE_NAME'"
echo ""
echo "The app should now appear properly in macOS notifications settings!"
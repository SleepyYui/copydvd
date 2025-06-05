#!/bin/bash

# Script to create a Linux installer for Copy DVD

set -e

APP_NAME="copy-dvd"
TARGET=${1:-"x86_64-unknown-linux-gnu"}
BINARY_PATH="target/$TARGET/release/copydvd"

# Extract version from Cargo.toml
VERSION=$(grep "^version = " Cargo.toml | sed 's/version = "\(.*\)"/\1/')

echo "Creating Linux installer for $APP_NAME..."

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    exit 1
fi

# Create installer directory structure
INSTALLER_DIR="installer-linux"
APP_DIR="$INSTALLER_DIR/opt/$APP_NAME"
BIN_DIR="$INSTALLER_DIR/usr/local/bin"
DESKTOP_DIR="$INSTALLER_DIR/usr/share/applications"
ICON_DIR="$INSTALLER_DIR/usr/share/icons/hicolor"

rm -rf "$INSTALLER_DIR"
mkdir -p "$APP_DIR" "$BIN_DIR" "$DESKTOP_DIR"

# Copy binary and resources
cp "$BINARY_PATH" "$APP_DIR/copydvd"
chmod +x "$APP_DIR/copydvd"

if [ -d "resources" ]; then
    cp -r "resources" "$APP_DIR/"
fi

# Create symlink for global access
ln -sf "/opt/$APP_NAME/copydvd" "$BIN_DIR/copydvd"

# Create desktop entry
cat > "$DESKTOP_DIR/$APP_NAME.desktop" << EOF
[Desktop Entry]
Version=1.0
Type=Application
Name=Copy DVD
Comment=DVD copying application with GUI and CLI support
Exec=/opt/$APP_NAME/copydvd
Icon=$APP_NAME
Terminal=false
Categories=AudioVideo;Video;
Keywords=DVD;copy;rip;HandBrake;
EOF

# Copy icons if they exist
if [ -d "resources/icons" ]; then
    for size in 16 32 64 128 256 512; do
        if [ -f "resources/icons/icon-${size}.png" ]; then
            mkdir -p "$ICON_DIR/${size}x${size}/apps"
            cp "resources/icons/icon-${size}.png" "$ICON_DIR/${size}x${size}/apps/$APP_NAME.png"
        fi
    done
fi

# Create install script
cat > "$INSTALLER_DIR/install.sh" << 'EOF'
#!/bin/bash

set -e

APP_NAME="copy-dvd"

if [ "$EUID" -ne 0 ]; then
    echo "Please run this installer as root (use sudo)"
    exit 1
fi

echo "Installing $APP_NAME..."

# Copy files
cp -r "$(dirname "$0")/opt/$APP_NAME" /opt/
cp -r "$(dirname "$0")/usr/"* /usr/

# Update desktop database
if command -v update-desktop-database > /dev/null; then
    update-desktop-database /usr/share/applications
fi

# Update icon cache
if command -v gtk-update-icon-cache > /dev/null; then
    gtk-update-icon-cache /usr/share/icons/hicolor
fi

echo "$APP_NAME has been installed successfully!"
echo "You can now run it with: copydvd"
echo "Or search for 'Copy DVD' in your application menu"
EOF

chmod +x "$INSTALLER_DIR/install.sh"

# Create uninstall script
cat > "$INSTALLER_DIR/uninstall.sh" << 'EOF'
#!/bin/bash

set -e

APP_NAME="copy-dvd"

if [ "$EUID" -ne 0 ]; then
    echo "Please run this uninstaller as root (use sudo)"
    exit 1
fi

echo "Uninstalling $APP_NAME..."

# Remove files
rm -rf "/opt/$APP_NAME"
rm -f "/usr/local/bin/copydvd"
rm -f "/usr/share/applications/$APP_NAME.desktop"
rm -rf "/usr/share/icons/hicolor/*/apps/$APP_NAME.png"

# Update desktop database
if command -v update-desktop-database > /dev/null; then
    update-desktop-database /usr/share/applications
fi

# Update icon cache
if command -v gtk-update-icon-cache > /dev/null; then
    gtk-update-icon-cache /usr/share/icons/hicolor
fi

echo "$APP_NAME has been uninstalled successfully!"
EOF

chmod +x "$INSTALLER_DIR/uninstall.sh"

# Create README
cat > "$INSTALLER_DIR/README.txt" << EOF
# $APP_NAME Installer v$VERSION

This is a simple installer for Copy DVD on Linux.

## Installation
1. Run: sudo ./install.sh
2. The application will be installed to /opt/$APP_NAME/

## Usage
- Run the GUI: copydvd
- Run the CLI: copydvd --cli
- Or search for "Copy DVD" in your application menu

## Uninstallation
Run: sudo ./uninstall.sh

## Requirements
- Linux with GUI support (for GUI mode)
- HandBrake CLI (automatically installed by the app)

EOF

echo "Linux installer created in: $INSTALLER_DIR"
echo "To install, run: sudo $INSTALLER_DIR/install.sh"
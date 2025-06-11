#!/bin/bash

# Build script to create a proper macOS app bundle for Copy DVD

set -e

APP_NAME="CopyDVD"
BUNDLE_NAME="CopyDVD.app"
BUNDLE_ID="com.sleepyyui.copydvd"
TARGET=${1:-"x86_64-apple-darwin"}

# Use VERSION from environment if available, otherwise extract from Cargo.toml
if [[ -z "$VERSION" ]]; then
    VERSION=$(grep "^version = " Cargo.toml | sed 's/version = "\(.*\)"/\1/')
fi

echo "Building Copy DVD app bundle for target: $TARGET"

# Build the release binary (skip if binary already exists from CI)
if [ ! -f "target/$TARGET/release/copydvd" ]; then
    echo "Building release binary..."
    cargo build --release --target $TARGET
fi

# Create app bundle structure
echo "Creating app bundle structure..."
mkdir -p "$BUNDLE_NAME/Contents/MacOS"
mkdir -p "$BUNDLE_NAME/Contents/Resources"

# Copy the binary
echo "Copying binary..."
cp "target/$TARGET/release/copydvd" "$BUNDLE_NAME/Contents/MacOS/"

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

# Fix permissions for the entire bundle
find "$BUNDLE_NAME" -type f -exec chmod 644 {} \;
find "$BUNDLE_NAME" -type d -exec chmod 755 {} \;
chmod +x "$BUNDLE_NAME/Contents/MacOS/copydvd"

# Remove quarantine attributes (required for distribution)
echo "Removing quarantine attributes..."
xattr -cr "$BUNDLE_NAME" 2>/dev/null || true

# Create a more comprehensive signing approach
if command -v codesign >/dev/null 2>&1; then
    echo "Attempting to sign the app bundle..."
    
    # First, sign the binary itself
    codesign --force --options runtime --sign - "$BUNDLE_NAME/Contents/MacOS/copydvd" 2>/dev/null || echo "Binary signing failed"
    
    # Then sign the entire app bundle with hardened runtime
    codesign --force --options runtime --deep --sign - "$BUNDLE_NAME" 2>/dev/null || echo "App bundle signing failed"
    
    # Verify the signature
    if codesign --verify --deep --strict "$BUNDLE_NAME" 2>/dev/null; then
        echo "✅ App bundle successfully signed and verified"
    else
        echo "⚠️ App signing verification failed (expected in CI)"
    fi
else
    echo "⚠️ codesign not available"
fi

# Add executable bit to the app bundle itself (required for proper launch)
chmod +x "$BUNDLE_NAME"

echo "App bundle created successfully: $BUNDLE_NAME"

# Create a proper macOS installer package (.pkg) with GUI
echo ""
echo "📦 Creating macOS installer package with GUI..."

PKG_NAME="CopyDVD-${TARGET}-${VERSION}.pkg"
PKG_TEMP_DIR="pkg_temp"
PKG_ROOT="$PKG_TEMP_DIR/root"
PKG_SCRIPTS="$PKG_TEMP_DIR/scripts"

# Clean up any existing temp directory
rm -rf "$PKG_TEMP_DIR"
mkdir -p "$PKG_ROOT/Applications"
mkdir -p "$PKG_SCRIPTS"

# Copy the app bundle to the package root
cp -R "$BUNDLE_NAME" "$PKG_ROOT/Applications/"

# Create preinstall script
cat > "$PKG_SCRIPTS/preinstall" << 'EOF'
#!/bin/bash
# Preinstall script for CopyDVD

# Remove any existing installation
if [ -d "/Applications/CopyDVD.app" ]; then
    echo "Removing existing CopyDVD installation..."
    rm -rf "/Applications/CopyDVD.app"
fi

exit 0
EOF

# Create postinstall script
cat > "$PKG_SCRIPTS/postinstall" << 'EOF'
#!/bin/bash
# Postinstall script for CopyDVD

# Set proper permissions
echo "Setting permissions for CopyDVD..."
chown -R root:admin "/Applications/CopyDVD.app"
chmod -R 755 "/Applications/CopyDVD.app"
chmod +x "/Applications/CopyDVD.app/Contents/MacOS/copydvd"

# Remove quarantine attributes
echo "Removing quarantine attributes..."
xattr -cr "/Applications/CopyDVD.app" 2>/dev/null || true

# Try to sign the installed app (will only work if codesign is available)
if command -v codesign >/dev/null 2>&1; then
    echo "Attempting to sign installed app..."
    codesign --force --options runtime --deep --sign - "/Applications/CopyDVD.app" 2>/dev/null || true
fi

echo "CopyDVD installation completed successfully!"
echo "You can find CopyDVD in your Applications folder."

exit 0
EOF

# Make scripts executable
chmod +x "$PKG_SCRIPTS/preinstall"
chmod +x "$PKG_SCRIPTS/postinstall"

# Create Distribution file for installer customization
cat > "$PKG_TEMP_DIR/Distribution.xml" << EOF
<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>CopyDVD ${VERSION}</title>
    <welcome file="welcome.html"/>
    <license file="license.txt"/>
    <conclusion file="conclusion.html"/>
    <domains enable_currentUserHome="false" enable_localSystem="true"/>
    <options customize="never" require-scripts="false" rootVolumeOnly="true"/>
    
    <choices-outline>
        <line choice="copydvd"/>
    </choices-outline>
    
    <choice id="copydvd" title="CopyDVD Application">
        <description>Install CopyDVD - A powerful DVD ripping application for macOS</description>
        <pkg-ref id="com.sleepyyui.copydvd.pkg"/>
    </choice>
    
    <pkg-ref id="com.sleepyyui.copydvd.pkg" version="$VERSION" onConclusion="none">copydvd-component.pkg</pkg-ref>
</installer-gui-script>
EOF

# Create welcome.html
cat > "$PKG_TEMP_DIR/welcome.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 20px; }
        h1 { color: #007AFF; }
        .feature { margin: 10px 0; }
    </style>
</head>
<body>
    <h1>Welcome to CopyDVD</h1>
    <p>This installer will install CopyDVD on your Mac.</p>
    
    <h2>Features:</h2>
    <div class="feature">• Easy DVD ripping with GUI and CLI interfaces</div>
    <div class="feature">• High-quality video transcoding with HandBrake</div>
    <div class="feature">• Multi-threaded processing for faster conversions</div>
    <div class="feature">• Upload support for remote servers</div>
    <div class="feature">• Cross-platform compatibility</div>
    
    <h2>System Requirements:</h2>
    <div class="feature">• macOS 10.15 (Catalina) or later</div>
    <div class="feature">• 4GB RAM minimum, 8GB recommended</div>
    <div class="feature">• DVD drive or mounted ISO files</div>
</body>
</html>
EOF

# Create license.txt
cat > "$PKG_TEMP_DIR/license.txt" << 'EOF'
CopyDVD License Agreement

This software is provided for educational purposes only.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

By installing this software, you acknowledge that you understand and agree to
comply with all applicable laws and regulations regarding the use of this
software.
EOF

# Create conclusion.html
cat > "$PKG_TEMP_DIR/conclusion.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 20px; }
        h1 { color: #28CD41; }
        .important { background: #FFF3CD; padding: 10px; border-radius: 5px; margin: 10px 0; }
    </style>
</head>
<body>
    <h1>Installation Complete!</h1>
    <p>CopyDVD has been successfully installed on your Mac.</p>
    
    <div class="important">
        <h3>Important Security Note:</h3>
        <p>On first launch, macOS may show a security warning because this app is not signed with an Apple Developer certificate.</p>
        
        <p><strong>To run CopyDVD:</strong></p>
        <ol>
            <li>Go to Applications folder</li>
            <li>Right-click on CopyDVD</li>
            <li>Select "Open" from the menu</li>
            <li>Click "Open" in the security dialog</li>
        </ol>
        
        <p>After this one-time authorization, you can launch CopyDVD normally.</p>
    </div>
    
    <p>You can find CopyDVD in your Applications folder or Launchpad.</p>
    <p>Thank you for installing CopyDVD!</p>
</body>
</html>
EOF

# Build the component package first
echo "Building component package..."
pkgbuild --root "$PKG_ROOT" \
         --scripts "$PKG_SCRIPTS" \
         --identifier com.sleepyyui.copydvd \
         --version "$VERSION" \
         --install-location "/" \
         "$PKG_TEMP_DIR/copydvd-component.pkg"

# Check if component package was created successfully
if [[ ! -f "$PKG_TEMP_DIR/copydvd-component.pkg" ]]; then
    echo "❌ Component package creation failed"
    rm -rf "$PKG_TEMP_DIR"
    exit 1
fi

echo "Component package size: $(du -h "$PKG_TEMP_DIR/copydvd-component.pkg" | cut -f1)"

# Build the final installer package with GUI
echo "Building installer package: $PKG_NAME"
productbuild --distribution "$PKG_TEMP_DIR/Distribution.xml" \
             --resources "$PKG_TEMP_DIR" \
             --package-path "$PKG_TEMP_DIR" \
             "$PKG_NAME"

if [[ -f "$PKG_NAME" ]]; then
    echo "✅ macOS installer package created: $PKG_NAME"
    echo "Size: $(du -h "$PKG_NAME" | cut -f1)"
    
    # Clean up temp directory
    rm -rf "$PKG_TEMP_DIR"
    
    echo ""
    echo "🍎 macOS Installation Instructions:"
    echo "1. Download the .pkg installer file"
    echo "2. Double-click to launch the installer"
    echo "3. Follow the guided installation process"
    echo "4. The installer will automatically place CopyDVD in Applications"
    echo "5. On first launch: Right-click CopyDVD → Open → Click 'Open'"
else
    echo "❌ Failed to create installer package"
    rm -rf "$PKG_TEMP_DIR"
fi

echo ""
echo "The app should now appear properly in macOS notifications settings!"
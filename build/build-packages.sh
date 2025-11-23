#!/bin/bash
# Cross-platform package builder for QuickSky

set -e

VERSION="0.1.0"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_DIR="$PROJECT_ROOT/build"
BINARY_PATH="$PROJECT_ROOT/target/release/sky"

echo "🏗️  Building QuickSky packages v$VERSION"

# Ensure binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo "❌ Binary not found. Run 'cargo build --release' first"
    exit 1
fi

# Build macOS PKG
build_macos_pkg() {
    echo "📦 Building macOS PKG..."
    
    PKG_ROOT="$BUILD_DIR/macos/pkg-root"
    mkdir -p "$PKG_ROOT/usr/local/bin"
    mkdir -p "$PKG_ROOT/usr/local/share/man/man1"
    
    # Copy binary and docs
    cp "$BINARY_PATH" "$PKG_ROOT/usr/local/bin/"
    cp "$PROJECT_ROOT/docs/sky.1" "$PKG_ROOT/usr/local/share/man/man1/"
    
    # Create PKG
    pkgbuild --root "$PKG_ROOT" \
             --identifier "com.saladin5101.quicksky" \
             --version "$VERSION" \
             --install-location "/" \
             "$BUILD_DIR/QuickSky-$VERSION.pkg"
    
    echo "✅ macOS PKG created: QuickSky-$VERSION.pkg"
}

# Build Windows MSI (requires WiX toolset)
build_windows_msi() {
    echo "📦 Building Windows MSI..."
    
    # Create WiX source
    cat > "$BUILD_DIR/windows/quicksky.wxs" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="QuickSky" Language="1033" Version="0.1.0" 
           Manufacturer="Saladin5101" UpgradeCode="12345678-1234-1234-1234-123456789012">
    <Package InstallerVersion="200" Compressed="yes" InstallScope="perMachine" />
    
    <MajorUpgrade DowngradeErrorMessage="A newer version is already installed." />
    <MediaTemplate EmbedCab="yes" />
    
    <Feature Id="ProductFeature" Title="QuickSky" Level="1">
      <ComponentGroupRef Id="ProductComponents" />
    </Feature>
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="QuickSky" />
      </Directory>
    </Directory>
    
    <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
      <Component Id="MainExecutable">
        <File Id="SkyExe" Source="sky.exe" KeyPath="yes" />
        <Environment Id="PATH" Name="PATH" Value="[INSTALLFOLDER]" 
                     Permanent="no" Part="last" Action="set" System="yes" />
      </Component>
    </ComponentGroup>
  </Product>
</Wix>
EOF
    
    # Copy binary (would need Windows cross-compilation)
    echo "⚠️  Windows MSI requires WiX toolset and Windows cross-compilation"
    echo "   Run this on Windows with: candle quicksky.wxs && light quicksky.wixobj"
}

# Build DEB package
build_debian_deb() {
    echo "📦 Building Debian DEB..."
    
    DEB_ROOT="$BUILD_DIR/debian/quicksky_$VERSION"
    mkdir -p "$DEB_ROOT/DEBIAN"
    mkdir -p "$DEB_ROOT/usr/bin"
    mkdir -p "$DEB_ROOT/usr/share/man/man1"
    mkdir -p "$DEB_ROOT/usr/share/info"
    
    # Copy files
    cp "$BINARY_PATH" "$DEB_ROOT/usr/bin/"
    cp "$PROJECT_ROOT/docs/sky.1" "$DEB_ROOT/usr/share/man/man1/"
    cp "$PROJECT_ROOT/docs/sky.info" "$DEB_ROOT/usr/share/info/"
    
    # Create control file
    cat > "$DEB_ROOT/DEBIAN/control" << EOF
Package: quicksky
Version: $VERSION
Section: devel
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.17)
Maintainer: Saladin5101 <saladin5101@example.com>
Description: Lazy developer-friendly native version control tool
 QuickSky is a no-bullshit, standalone version control tool built from scratch
 to fix the stuff that makes version control feel like a chore.
 .
 Features:
  * One-click upload with sky upload
  * Zero-hassle repository switching  
  * Pain-free rebasing by date range
  * Simple branch management
  * Native implementation with no Git dependencies
Homepage: https://github.com/Saladin5101/QuickSky
EOF
    
    # Create postinst script
    cat > "$DEB_ROOT/DEBIAN/postinst" << 'EOF'
#!/bin/bash
# Update man database
if command -v mandb >/dev/null 2>&1; then
    mandb -q
fi
# Update info directory
if command -v install-info >/dev/null 2>&1; then
    install-info --dir-file=/usr/share/info/dir /usr/share/info/sky.info
fi
EOF
    chmod 755 "$DEB_ROOT/DEBIAN/postinst"
    
    # Build DEB
    dpkg-deb --build "$DEB_ROOT" "$BUILD_DIR/quicksky_$VERSION-1_amd64.deb"
    
    echo "✅ Debian DEB created: quicksky_$VERSION-1_amd64.deb"
}

# Main build process
case "${1:-all}" in
    "macos")
        build_macos_pkg
        ;;
    "windows") 
        build_windows_msi
        ;;
    "debian")
        build_debian_deb
        ;;
    "all")
        if [[ "$OSTYPE" == "darwin"* ]]; then
            build_macos_pkg
        fi
        build_debian_deb
        build_windows_msi
        ;;
    *)
        echo "Usage: $0 [macos|windows|debian|all]"
        exit 1
        ;;
esac

echo "🎉 Package build complete!"
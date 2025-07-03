#!/bin/bash
# this_file: scripts/build_linux.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building svgn for Linux...${NC}"

# Check if we're on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo -e "${RED}Error: This script must be run on Linux${NC}"
    exit 1
fi

# Setup paths
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
DIST_DIR="$PROJECT_ROOT/dist/linux"
CARGO_DIR="$PROJECT_ROOT/svgn"

# Clean and create dist directory
echo -e "${YELLOW}Creating distribution directory...${NC}"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# Install required tools if not present
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Determine architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        TARGET="x86_64-unknown-linux-gnu"
        ARCH_NAME="x86_64"
        ;;
    aarch64)
        TARGET="aarch64-unknown-linux-gnu"
        ARCH_NAME="aarch64"
        ;;
    armv7l)
        TARGET="armv7-unknown-linux-gnueabihf"
        ARCH_NAME="armv7"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${YELLOW}Building for $ARCH_NAME ($TARGET)...${NC}"

# Add target if not already added
rustup target add "$TARGET" 2>/dev/null || true

# Build release binary
cd "$CARGO_DIR"
cargo build --release --target "$TARGET"

# Copy binary to dist directory
cp "target/$TARGET/release/svgn" "$DIST_DIR/"
chmod +x "$DIST_DIR/svgn"

# Strip the binary to reduce size
echo -e "${YELLOW}Stripping binary...${NC}"
strip "$DIST_DIR/svgn"

# Verify the binary
echo -e "${YELLOW}Verifying binary...${NC}"
file "$DIST_DIR/svgn"
ldd "$DIST_DIR/svgn" 2>/dev/null || echo "Note: Binary might be statically linked"

# Test the binary
echo -e "${YELLOW}Testing binary...${NC}"
"$DIST_DIR/svgn" --version || {
    echo -e "${RED}Error: Binary test failed${NC}"
    exit 1
}

# Check binary size (warn if too large)
BINARY_SIZE=$(stat -c%s "$DIST_DIR/svgn" 2>/dev/null || stat -f%z "$DIST_DIR/svgn")
BINARY_SIZE_MB=$((BINARY_SIZE / 1024 / 1024))
if [ $BINARY_SIZE_MB -gt 50 ]; then
    echo -e "${YELLOW}Warning: Binary size is ${BINARY_SIZE_MB}MB, consider optimization${NC}"
fi

# Get version from Cargo.toml
VERSION=$(grep '^version' "$CARGO_DIR/Cargo.toml" | head -1 | cut -d '"' -f 2)
echo -e "${GREEN}Building version: $VERSION${NC}"

# Create tar.gz archive
echo -e "${YELLOW}Creating tar.gz archive...${NC}"
cd "$DIST_DIR"
tar -czf "svgn-$VERSION-linux-$ARCH_NAME.tar.gz" svgn
cd "$PROJECT_ROOT"

# Create .deb package if dpkg is available
if command -v dpkg &> /dev/null; then
    echo -e "${YELLOW}Creating .deb package...${NC}"
    
    DEB_DIR="$DIST_DIR/deb"
    mkdir -p "$DEB_DIR/DEBIAN"
    mkdir -p "$DEB_DIR/usr/local/bin"
    
    # Copy binary
    cp "$DIST_DIR/svgn" "$DEB_DIR/usr/local/bin/"
    
    # Create control file
    cat > "$DEB_DIR/DEBIAN/control" << EOF
Package: svgn
Version: $VERSION
Section: utils
Priority: optional
Architecture: $ARCH_NAME
Maintainer: svgn developers
Description: SVG Optimizer - A fast, native SVG optimization tool
 svgn is a Rust port of SVGO, providing high-performance SVG optimization
 with a command-line interface.
EOF
    
    # Build the .deb package
    dpkg-deb --build "$DEB_DIR" "$DIST_DIR/svgn-$VERSION-linux-$ARCH_NAME.deb"
    
    # Clean up
    rm -rf "$DEB_DIR"
else
    echo -e "${YELLOW}dpkg not found, skipping .deb package creation${NC}"
fi

# Create RPM package if rpmbuild is available
if command -v rpmbuild &> /dev/null; then
    echo -e "${YELLOW}Creating .rpm package...${NC}"
    
    RPM_DIR="$DIST_DIR/rpm"
    mkdir -p "$RPM_DIR/BUILD"
    mkdir -p "$RPM_DIR/RPMS"
    mkdir -p "$RPM_DIR/SOURCES"
    mkdir -p "$RPM_DIR/SPECS"
    
    # Create spec file
    cat > "$RPM_DIR/SPECS/svgn.spec" << EOF
Name:           svgn
Version:        $VERSION
Release:        1%{?dist}
Summary:        SVG Optimizer - A fast, native SVG optimization tool

License:        MIT
URL:            https://github.com/twardoch/svgn

%description
svgn is a Rust port of SVGO, providing high-performance SVG optimization
with a command-line interface.

%install
mkdir -p %{buildroot}/usr/local/bin
install -m 755 $DIST_DIR/svgn %{buildroot}/usr/local/bin/svgn

%files
/usr/local/bin/svgn

%changelog
* $(date +"%a %b %d %Y") svgn developers - $VERSION-1
- Release version $VERSION
EOF
    
    # Build the RPM
    rpmbuild --define "_topdir $RPM_DIR" \
             --define "_rpmdir $DIST_DIR" \
             -bb "$RPM_DIR/SPECS/svgn.spec" || echo -e "${YELLOW}RPM build failed${NC}"
    
    # Clean up
    rm -rf "$RPM_DIR"
else
    echo -e "${YELLOW}rpmbuild not found, skipping .rpm package creation${NC}"
fi

# Create AppImage if available
if command -v appimagetool &> /dev/null; then
    echo -e "${YELLOW}Creating AppImage...${NC}"
    
    APPIMAGE_DIR="$DIST_DIR/AppDir"
    mkdir -p "$APPIMAGE_DIR/usr/bin"
    
    # Copy binary
    cp "$DIST_DIR/svgn" "$APPIMAGE_DIR/usr/bin/"
    
    # Create desktop file
    cat > "$APPIMAGE_DIR/svgn.desktop" << EOF
[Desktop Entry]
Name=svgn
Exec=svgn
Icon=svgn
Type=Application
Categories=Utility;
Terminal=true
EOF
    
    # Create a simple icon (1x1 PNG)
    printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x02\x00\x00\x00\x90wS\xde\x00\x00\x00\x0cIDATx\x9cc\xf8\x0f\x00\x00\x01\x01\x00\x05T\xbc\xa8\x15\x00\x00\x00\x00IEND\xaeB`\x82' > "$APPIMAGE_DIR/svgn.png"
    
    # Create AppRun
    cat > "$APPIMAGE_DIR/AppRun" << 'EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "$HERE/usr/bin/svgn" "$@"
EOF
    chmod +x "$APPIMAGE_DIR/AppRun"
    
    # Build AppImage
    ARCH=$ARCH_NAME appimagetool "$APPIMAGE_DIR" "$DIST_DIR/svgn-$VERSION-linux-$ARCH_NAME.AppImage" || echo -e "${YELLOW}AppImage creation failed${NC}"
    
    # Clean up
    rm -rf "$APPIMAGE_DIR"
else
    echo -e "${YELLOW}appimagetool not found, skipping AppImage creation${NC}"
fi

# Create install script
cat > "$DIST_DIR/install.sh" << 'EOF'
#!/bin/bash
echo "Installing svgn to /usr/local/bin..."
sudo cp svgn /usr/local/bin/
sudo chmod +x /usr/local/bin/svgn
echo "Installation complete! Run 'svgn --help' to get started."
EOF
chmod +x "$DIST_DIR/install.sh"

# Summary
echo -e "${GREEN}Build complete!${NC}"
echo -e "${GREEN}Distribution files created in: $DIST_DIR${NC}"
echo -e "  - svgn (binary)"
echo -e "  - svgn-$VERSION-linux-$ARCH_NAME.tar.gz"
if [[ -f "$DIST_DIR/svgn-$VERSION-linux-$ARCH_NAME.deb" ]]; then
    echo -e "  - svgn-$VERSION-linux-$ARCH_NAME.deb"
fi
if [[ -f "$DIST_DIR/$ARCH_NAME/svgn-$VERSION-1.$ARCH_NAME.rpm" ]]; then
    echo -e "  - svgn-$VERSION-1.$ARCH_NAME.rpm"
fi
if [[ -f "$DIST_DIR/svgn-$VERSION-linux-$ARCH_NAME.AppImage" ]]; then
    echo -e "  - svgn-$VERSION-linux-$ARCH_NAME.AppImage"
fi
echo -e "  - install.sh"
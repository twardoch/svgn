#!/bin/bash
# this_file: scripts/build_macos.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building svgn for macOS...${NC}"

# Check if we're on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}Error: This script must be run on macOS${NC}"
    exit 1
fi

# Setup paths
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/.." && pwd )"
DIST_DIR="$PROJECT_ROOT/dist/macos"
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

# Add targets if not already added
echo -e "${YELLOW}Adding build targets...${NC}"
rustup target add x86_64-apple-darwin 2>/dev/null || true
rustup target add aarch64-apple-darwin 2>/dev/null || true

# Build for x86_64
echo -e "${YELLOW}Building for x86_64...${NC}"
cd "$CARGO_DIR"
cargo build --release --target x86_64-apple-darwin

# Build for aarch64 (Apple Silicon)
echo -e "${YELLOW}Building for aarch64...${NC}"
cargo build --release --target aarch64-apple-darwin

# Create universal binary
echo -e "${YELLOW}Creating universal binary...${NC}"
lipo -create \
    target/x86_64-apple-darwin/release/svgn \
    target/aarch64-apple-darwin/release/svgn \
    -output "$DIST_DIR/svgn"

# Make it executable
chmod +x "$DIST_DIR/svgn"

# Verify the universal binary
echo -e "${YELLOW}Verifying universal binary...${NC}"
file "$DIST_DIR/svgn"
lipo -info "$DIST_DIR/svgn"

# Test the binary
echo -e "${YELLOW}Testing binary...${NC}"
"$DIST_DIR/svgn" --version || {
    echo -e "${RED}Error: Binary test failed${NC}"
    exit 1
}

# Verify it's truly universal
ARCHS=$(lipo -archs "$DIST_DIR/svgn")
if [[ "$ARCHS" != *"x86_64"* ]] || [[ "$ARCHS" != *"arm64"* ]]; then
    echo -e "${RED}Error: Binary is not universal (found: $ARCHS)${NC}"
    exit 1
fi

# Get version from Cargo.toml
VERSION=$(grep '^version' "$CARGO_DIR/Cargo.toml" | head -1 | cut -d '"' -f 2)
echo -e "${GREEN}Building version: $VERSION${NC}"

# Create tar.gz archive
echo -e "${YELLOW}Creating tar.gz archive...${NC}"
cd "$DIST_DIR"
tar -czf "svgn-$VERSION-macos-universal.tar.gz" svgn
cd "$PROJECT_ROOT"

# Create pkg installer
echo -e "${YELLOW}Creating .pkg installer...${NC}"
PKG_ROOT="$DIST_DIR/pkg-root"
mkdir -p "$PKG_ROOT/usr/local/bin"
cp "$DIST_DIR/svgn" "$PKG_ROOT/usr/local/bin/"

# Create the package
pkgbuild \
    --root "$PKG_ROOT" \
    --identifier "com.svgn.cli" \
    --version "$VERSION" \
    --install-location "/" \
    "$DIST_DIR/svgn-$VERSION-macos.pkg"

# Clean up pkg root
rm -rf "$PKG_ROOT"

# Create DMG
echo -e "${YELLOW}Creating .dmg...${NC}"
DMG_ROOT="$DIST_DIR/dmg-root"
mkdir -p "$DMG_ROOT"
cp "$DIST_DIR/svgn" "$DMG_ROOT/"

# Create a simple README for the DMG
cat > "$DMG_ROOT/README.txt" << EOF
svgn - SVG Optimizer
Version: $VERSION

To install:
1. Copy 'svgn' to /usr/local/bin/ or another directory in your PATH
2. Make sure it's executable: chmod +x /usr/local/bin/svgn
3. Run: svgn --help

Or use the .pkg installer for automatic installation.
EOF

# Create DMG
hdiutil create -volname "svgn-$VERSION" \
    -srcfolder "$DMG_ROOT" \
    -ov \
    -format UDZO \
    "$DIST_DIR/svgn-$VERSION-macos.dmg"

# Clean up DMG root
rm -rf "$DMG_ROOT"

# Create a simple install script
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
echo -e "  - svgn (universal binary)"
echo -e "  - svgn-$VERSION-macos-universal.tar.gz"
echo -e "  - svgn-$VERSION-macos.pkg"
echo -e "  - svgn-$VERSION-macos.dmg"
echo -e "  - install.sh"
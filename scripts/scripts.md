# Build Scripts

This directory contains platform-specific build scripts for creating distribution packages of svgn.

## Scripts

### `build_macos.sh`
Builds svgn for macOS, creating:
- Universal binary (x86_64 + arm64)
- `.tar.gz` archive
- `.pkg` installer (installs to `/usr/local/bin`)
- `.dmg` disk image
- Install script

### `build_linux.sh`
Builds svgn for Linux, creating:
- Native binary for the current architecture
- `.tar.gz` archive
- `.deb` package (if dpkg is available)
- `.rpm` package (if rpmbuild is available)
- AppImage (if appimagetool is available)
- Install script

### `build_windows.cmd`
Builds svgn for Windows, creating:
- `.exe` executable
- `.zip` archive
- `.msi` installer (if WiX Toolset is available)
- Setup installer (if Inno Setup is available)
- Install batch script

## Usage

Each script should be run on its respective platform:

```bash
# On macOS
./scripts/build_macos.sh

# On Linux
./scripts/build_linux.sh

# On Windows
scripts\build_windows.cmd
```

## Output

All build artifacts are placed in:
- `dist/macos/` - macOS builds
- `dist/linux/` - Linux builds
- `dist/windows/` - Windows builds

## CI/CD Integration

These scripts are used by the GitHub Actions workflow in `.github/workflows/release.yml` to create release artifacts automatically when a new version tag is pushed.

## Requirements

### Common
- Rust toolchain with cargo
- Git

### macOS
- Xcode Command Line Tools (for lipo, pkgbuild)
- Both x86_64 and aarch64 Rust targets

### Linux
- Standard build tools
- Optional: dpkg-dev (for .deb packages)
- Optional: rpm-build (for .rpm packages)
- Optional: appimagetool (for AppImage)

### Windows
- Visual Studio Build Tools or Visual Studio
- Optional: WiX Toolset (for .msi installer)
- Optional: Inno Setup (for setup.exe installer)
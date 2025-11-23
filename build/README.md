# QuickSky Package Building

This directory contains scripts and configurations for building QuickSky installation packages across different platforms.

## Prerequisites

1. **Rust binary**: Run `cargo build --release` in project root
2. **Documentation**: Run `make info` in `docs/` directory  
3. **Platform tools**:
   - macOS: Xcode Command Line Tools (for `pkgbuild`)
   - Windows: WiX Toolset v3.x
   - Linux: `dpkg-deb` (usually pre-installed)

## Building Packages

### All Platforms
```bash
./build-packages.sh all
```

### Specific Platform
```bash
./build-packages.sh macos     # Creates .pkg
./build-packages.sh debian    # Creates .deb  
./build-packages.sh windows   # Creates .msi (Windows only)
```

## Package Outputs

- **macOS PKG**: `QuickSky-0.1.0.pkg`
  - Installs to `/usr/local/bin/sky`
  - Includes man page at `/usr/local/share/man/man1/sky.1`

- **Debian DEB**: `quicksky_0.1.0-1_amd64.deb`
  - Installs to `/usr/bin/sky`
  - Includes man page and info documentation
  - Auto-updates man/info databases

- **Windows MSI**: `QuickSky-0.1.0.msi`
  - Installs to `Program Files\QuickSky\sky.exe`
  - Adds to system PATH automatically

## Cross-Compilation Notes

For Windows MSI on non-Windows systems:
1. Use `cargo build --target x86_64-pc-windows-gnu --release`
2. Copy `target/x86_64-pc-windows-gnu/release/sky.exe` to `build/windows/`
3. Run WiX tools on Windows system

## Testing Packages

### macOS PKG
```bash
sudo installer -pkg QuickSky-0.1.0.pkg -target /
sky --help
```

### Debian DEB  
```bash
sudo dpkg -i quicksky_0.1.0-1_amd64.deb
sky --help
man sky
```

### Windows MSI
Double-click to install, then:
```cmd
sky --help
```

## Package Contents

All packages include:
- `sky` binary (platform-specific)
- Man page (`sky.1`)
- Info documentation (`sky.info` - Linux only)
- Automatic PATH setup
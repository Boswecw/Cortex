# Cortex Deployment Guide

**Version:** 0.1.0 (Phase 0)
**Last Updated:** 2025-11-29

---

## Table of Contents

1. [Overview](#overview)
2. [Building for Production](#building-for-production)
3. [Platform-Specific Builds](#platform-specific-builds)
4. [Distribution](#distribution)
5. [CI/CD Setup](#cicd-setup)
6. [Release Process](#release-process)
7. [Troubleshooting](#troubleshooting)

---

## Overview

Cortex is built with Tauri, which produces platform-native applications:
- **Linux:** `.deb`, `.AppImage`
- **macOS:** `.dmg`, `.app`
- **Windows:** `.msi`, `.exe`

### Build Matrix

| Platform | Artifact | Size (approx) | Target |
|----------|----------|---------------|--------|
| Linux x64 | `.deb` | ~15MB | Ubuntu 20.04+ |
| Linux x64 | `.AppImage` | ~20MB | Any distro |
| macOS x64 | `.dmg` | ~10MB | macOS 10.15+ |
| macOS ARM | `.dmg` | ~10MB | Apple Silicon |
| Windows x64 | `.msi` | ~12MB | Windows 10+ |

---

## Building for Production

### Prerequisites

**All Platforms:**
- Rust 1.75+
- Node.js 18+
- npm 9+

**Platform-Specific:**
See [Platform-Specific Builds](#platform-specific-builds).

### Build Steps

**1. Clean Build Environment:**
```bash
# Remove old builds
rm -rf src-tauri/target/release
rm -rf .svelte-kit
rm -rf build

# Clean dependencies
cd src-tauri && cargo clean && cd ..
```

**2. Install Dependencies:**
```bash
npm install
cd src-tauri && cargo fetch && cd ..
```

**3. Build Frontend:**
```bash
npm run build
```

Verify output in `build/` directory.

**4. Build Backend:**
```bash
cd src-tauri
cargo build --release
```

**5. Build Tauri Application:**
```bash
npm run tauri build
```

**Output Location:**
- Bundles: `src-tauri/target/release/bundle/`
- Binary: `src-tauri/target/release/cortex` (or `cortex.exe`)

### Build Configuration

**Cargo.toml:**
```toml
[profile.release]
opt-level = 3         # Maximum optimization
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization
strip = true          # Remove symbols (smaller binary)
panic = 'abort'       # Smaller binary
```

**tauri.conf.json:**
```json
{
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devPath": "http://localhost:1420",
    "distDir": "../build"
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage"],
    "identifier": "com.cortex.app",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  }
}
```

---

## Platform-Specific Builds

### Linux

**System Requirements:**
- Ubuntu 20.04+ or equivalent
- glibc 2.31+

**Dependencies:**
```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  build-essential
```

**Build:**
```bash
npm run tauri build
```

**Outputs:**
- `.deb`: `src-tauri/target/release/bundle/deb/cortex_0.1.0_amd64.deb`
- `.AppImage`: `src-tauri/target/release/bundle/appimage/cortex_0.1.0_amd64.AppImage`

**Install .deb:**
```bash
sudo dpkg -i cortex_0.1.0_amd64.deb
sudo apt-get install -f  # Fix dependencies
```

**Run AppImage:**
```bash
chmod +x cortex_0.1.0_amd64.AppImage
./cortex_0.1.0_amd64.AppImage
```

**Uninstall:**
```bash
sudo apt-get remove cortex
```

### macOS

**System Requirements:**
- macOS 10.15 Catalina or later
- Xcode Command Line Tools

**Dependencies:**
```bash
xcode-select --install
```

**Build for Intel:**
```bash
rustup target add x86_64-apple-darwin
npm run tauri build -- --target x86_64-apple-darwin
```

**Build for Apple Silicon:**
```bash
rustup target add aarch64-apple-darwin
npm run tauri build -- --target aarch64-apple-darwin
```

**Build Universal Binary:**
```bash
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
npm run tauri build -- --target universal-apple-darwin
```

**Outputs:**
- `.dmg`: `src-tauri/target/release/bundle/dmg/Cortex_0.1.0_x64.dmg`
- `.app`: `src-tauri/target/release/bundle/macos/Cortex.app`

**Code Signing (Required for Distribution):**
```bash
# Get your Apple Developer ID
security find-identity -v -p codesigning

# Sign application
codesign --force --deep --sign "Developer ID Application: Your Name" \
  src-tauri/target/release/bundle/macos/Cortex.app

# Verify signature
codesign --verify --deep --strict --verbose=2 \
  src-tauri/target/release/bundle/macos/Cortex.app

# Create notarized DMG
xcrun notarytool submit Cortex_0.1.0_x64.dmg \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAMID"
```

**Install:**
- Mount DMG
- Drag Cortex.app to Applications folder

**Uninstall:**
```bash
rm -rf /Applications/Cortex.app
rm -rf ~/Library/Application\ Support/com.cortex.app
```

### Windows

**System Requirements:**
- Windows 10 or later
- Visual Studio 2022 or Build Tools

**Dependencies:**
```powershell
# Install Visual Studio Build Tools 2022
# Or full Visual Studio 2022 with C++ tools
```

**Build:**
```powershell
npm run tauri build
```

**Outputs:**
- `.msi`: `src-tauri\target\release\bundle\msi\Cortex_0.1.0_x64_en-US.msi`
- `.exe`: `src-tauri\target\release\cortex.exe`

**Code Signing (Optional):**
```powershell
# Get code signing certificate
# Sign MSI
signtool sign /f certificate.pfx /p password /tr http://timestamp.digicert.com /td sha256 /fd sha256 Cortex_0.1.0_x64_en-US.msi
```

**Install:**
- Double-click MSI
- Follow installer wizard

**Uninstall:**
- Control Panel → Programs → Uninstall Cortex
- Or: `msiexec /x {PRODUCT_CODE}`

---

## Distribution

### GitHub Releases

**Manual Release:**

1. **Create Git Tag:**
```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

2. **Build All Platforms:**
```bash
# Linux
npm run tauri build

# Upload artifacts to GitHub Releases
```

3. **Create Release on GitHub:**
- Go to Releases → Draft new release
- Tag: `v0.1.0`
- Title: `Cortex v0.1.0`
- Description: Changelog
- Attach build artifacts

**Automated Release with GitHub Actions:**
See [CI/CD Setup](#cicd-setup).

### Checksums

**Generate checksums:**
```bash
cd src-tauri/target/release/bundle

# Linux
sha256sum deb/cortex_0.1.0_amd64.deb > checksums.txt
sha256sum appimage/cortex_0.1.0_amd64.AppImage >> checksums.txt

# macOS
shasum -a 256 dmg/Cortex_0.1.0_x64.dmg >> checksums.txt

# Windows (PowerShell)
Get-FileHash msi\Cortex_0.1.0_x64_en-US.msi -Algorithm SHA256 >> checksums.txt
```

**Include in release:**
```markdown
## Downloads

| Platform | File | SHA256 |
|----------|------|--------|
| Linux .deb | [cortex_0.1.0_amd64.deb] | `abc123...` |
| Linux AppImage | [cortex_0.1.0_amd64.AppImage] | `def456...` |
| macOS Intel | [Cortex_0.1.0_x64.dmg] | `ghi789...` |
| Windows | [Cortex_0.1.0_x64_en-US.msi] | `jkl012...` |
```

### Packaging Managers

**Future: Homebrew (macOS/Linux):**
```bash
brew install cortex
```

**Future: Chocolatey (Windows):**
```powershell
choco install cortex
```

**Future: AUR (Arch Linux):**
```bash
yay -S cortex
```

---

## CI/CD Setup

### GitHub Actions

**.github/workflows/release.yml:**
```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-linux:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 18

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev \
            libappindicator3-dev librsvg2-dev patchelf \
            libssl-dev libgtk-3-dev

      - name: Install Node dependencies
        run: npm install

      - name: Build
        run: npm run tauri build

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: linux-build
          path: |
            src-tauri/target/release/bundle/deb/*.deb
            src-tauri/target/release/bundle/appimage/*.AppImage

  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 18

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: universal-apple-darwin

      - name: Install dependencies
        run: npm install

      - name: Build
        run: npm run tauri build -- --target universal-apple-darwin

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: macos-build
          path: src-tauri/target/release/bundle/dmg/*.dmg

  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 18

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install dependencies
        run: npm install

      - name: Build
        run: npm run tauri build

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: windows-build
          path: src-tauri/target/release/bundle/msi/*.msi

  release:
    needs: [build-linux, build-macos, build-windows]
    runs-on: ubuntu-latest
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v3

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            linux-build/**
            macos-build/**
            windows-build/**
          draft: false
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Testing Workflow

**.github/workflows/test.yml:**
```yaml
name: Test

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run tests
        run: |
          cd src-tauri
          cargo test --lib
          cargo test --tests

      - name: Run clippy
        run: |
          cd src-tauri
          cargo clippy -- -D warnings

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 18

      - name: Install dependencies
        run: npm install

      - name: Run tests
        run: npm test

      - name: Build check
        run: npm run build
```

---

## Release Process

### Pre-Release Checklist

- [ ] All tests passing (`cargo test && npm test`)
- [ ] Benchmarks meet targets (`cargo run --release --bin load_test`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version bumped in:
  - [ ] `src-tauri/Cargo.toml`
  - [ ] `src-tauri/tauri.conf.json`
  - [ ] `package.json`
- [ ] Icons updated (if changed)
- [ ] Code signed (macOS/Windows if distributing)

### Version Numbering

Follow [Semantic Versioning](https://semver.org/):

**Format:** `MAJOR.MINOR.PATCH`

**Examples:**
- `0.1.0` - Initial Phase 0 release
- `0.1.1` - Bug fix
- `0.2.0` - New features (Phase 1)
- `1.0.0` - Stable release

**Pre-releases:**
- `0.1.0-alpha.1` - Alpha version
- `0.1.0-beta.1` - Beta version
- `0.1.0-rc.1` - Release candidate

### Release Steps

**1. Update Version:**

```bash
# Update all version files
./scripts/bump-version.sh 0.1.0
```

Or manually:

```toml
# src-tauri/Cargo.toml
[package]
version = "0.1.0"
```

```json
// package.json
{
  "version": "0.1.0"
}
```

```json
// src-tauri/tauri.conf.json
{
  "package": {
    "version": "0.1.0"
  }
}
```

**2. Update CHANGELOG:**

```markdown
# Changelog

## [0.1.0] - 2025-11-29

### Added
- Full-text search with FTS5
- File indexing pipeline
- Advanced search filters
- Real-time indexing progress

### Fixed
- Database locking issues
- Memory leaks in extraction

### Changed
- Improved search performance (5x faster)
- Updated UI theme

### Removed
- Deprecated API endpoints
```

**3. Commit and Tag:**

```bash
git add .
git commit -m "chore: bump version to 0.1.0"
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin main
git push origin v0.1.0
```

**4. Build Release Artifacts:**

```bash
# Clean build
npm run tauri build

# Generate checksums
cd src-tauri/target/release/bundle
find . -type f \( -name "*.deb" -o -name "*.AppImage" -o -name "*.dmg" -o -name "*.msi" \) -exec sha256sum {} \; > checksums.txt
```

**5. Create GitHub Release:**

- Go to GitHub → Releases → Draft new release
- Tag: `v0.1.0`
- Title: `Cortex v0.1.0`
- Copy CHANGELOG entry to description
- Upload artifacts
- Upload checksums.txt
- Publish release

**6. Announce:**

- Post to Discord/community channels
- Update website (if applicable)
- Social media announcement

### Post-Release

- [ ] Verify downloads work
- [ ] Test installation on all platforms
- [ ] Monitor issue tracker for bug reports
- [ ] Update documentation site

---

## Troubleshooting

### Build Errors

**Problem: "WebKit2GTK not found"**

**Solution:**
```bash
sudo apt-get install libwebkit2gtk-4.1-dev
```

**Problem: "Bundle identifier collision"**

**Solution:**
Update `bundle.identifier` in `tauri.conf.json`:
```json
{
  "bundle": {
    "identifier": "com.yourcompany.cortex"
  }
}
```

**Problem: "Icon not found"**

**Solution:**
Generate icons:
```bash
# Install icon generator
npm install -g @tauri-apps/cli

# Generate from PNG
tauri icon path/to/icon.png
```

### Signing Issues

**macOS: "Code signature invalid"**

**Solutions:**
```bash
# Re-sign with correct identity
codesign --force --deep --sign "Developer ID" Cortex.app

# Verify
codesign --verify --deep --strict Cortex.app
spctl -a -vv Cortex.app
```

**Windows: "Untrusted publisher"**

**Solutions:**
- Get code signing certificate from trusted CA
- Sign with `signtool`
- Users may need to approve first run

### Distribution Issues

**Problem: "App won't open on macOS"**

**Causes:**
- Gatekeeper blocking unsigned app
- Missing quarantine attribute

**Solutions:**
```bash
# Remove quarantine
xattr -d com.apple.quarantine /Applications/Cortex.app

# Or sign and notarize properly
```

**Problem: ".deb install fails"**

**Solution:**
```bash
# Check dependencies
dpkg -I cortex_0.1.0_amd64.deb

# Install missing deps
sudo apt-get install -f
```

**Problem: "MSI won't install on Windows"**

**Causes:**
- Insufficient permissions
- Previous version not uninstalled

**Solutions:**
- Run as Administrator
- Uninstall old version first
- Check Windows Event Viewer for errors

---

## Optimization

### Binary Size Reduction

**Cargo.toml:**
```toml
[profile.release]
strip = true          # Remove symbols
lto = true            # Link-time optimization
codegen-units = 1     # Better optimization
opt-level = "z"       # Optimize for size (instead of "3" for speed)
```

**Remove unused dependencies:**
```bash
cargo tree --duplicates
cargo bloat --release
```

**Expected sizes:**
- Unoptimized: ~50MB
- Optimized: ~15MB
- With UPX: ~8MB (not recommended for Tauri)

### Performance Optimization

**Frontend:**
- Enable minification in Vite
- Tree-shaking enabled
- Code splitting by route

**Backend:**
- Profile-guided optimization (PGO)
- CPU-specific optimizations
- Static linking

---

## Security

### Code Signing

**Why sign?**
- Users trust signed applications
- Required for auto-updates
- Prevents tampering

**Certificates needed:**
- **macOS:** Apple Developer ID Application certificate
- **Windows:** Code signing certificate from CA (DigiCert, Sectigo, etc.)

### Secure Release Process

1. Build on clean, trusted environment
2. Sign all artifacts
3. Generate checksums
4. Use HTTPS for downloads
5. Verify signatures in auto-update

### Update Security

**Future: Tauri Updater:**
```json
{
  "updater": {
    "active": true,
    "endpoints": [
      "https://releases.cortex.app/{{target}}/{{current_version}}"
    ],
    "pubkey": "YOUR_PUBLIC_KEY"
  }
}
```

---

## Additional Resources

- [Tauri Building Guide](https://tauri.app/v2/guides/building/)
- [Code Signing Guide](https://tauri.app/v2/guides/distribution/sign-macos/)
- [GitHub Actions for Tauri](https://tauri.app/v2/guides/building/cross-platform/)
- [Semantic Versioning](https://semver.org/)

---

**Cortex Deployment Guide v0.1.0** | [Report Issue](https://github.com/yourusername/cortex/issues) | [Edit on GitHub](https://github.com/yourusername/cortex/edit/main/docs/DEPLOYMENT.md)

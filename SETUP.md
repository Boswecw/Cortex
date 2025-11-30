# Cortex Setup Guide

## System Requirements

### Linux (Ubuntu/Debian)

Before building Cortex on Linux, install the required system dependencies:

```bash
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl-dev \
    libgtk-3-dev
```

### macOS

```bash
# Xcode Command Line Tools are required
xcode-select --install
```

### Windows

No additional system dependencies required.

## Development Setup

1. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Install Node.js** (v18+)
   ```bash
   # Using nvm
   nvm install 18
   nvm use 18
   ```

3. **Clone and setup**
   ```bash
   cd cortex
   npm install
   ```

4. **Build and run**
   ```bash
   # Development mode
   npm run dev

   # Production build
   npm run build
   ```

## Testing Backend Only

If you want to test just the Rust backend without the GUI:

```bash
cd src-tauri
cargo test
```

## Troubleshooting

### Linux: Missing system libraries

If you get errors about missing `glib-2.0.pc` or similar:
- Make sure you've installed all dependencies listed above
- Check `PKG_CONFIG_PATH` includes `/usr/lib/pkgconfig`

### WSL2 specific

For WSL2, you may need to install an X server to run the GUI:
- Install VcXsrv or X410
- Set `DISPLAY` environment variable

Alternatively, build and test the backend only using `cargo test`.

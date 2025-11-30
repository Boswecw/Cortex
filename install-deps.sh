#!/bin/bash
# Cortex System Dependencies Installation Script
# This script installs required Linux system libraries for Tauri development

set -e  # Exit on error

echo "================================="
echo "Cortex Dependency Installation"
echo "================================="
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "This script requires sudo privileges."
    echo "You may be prompted for your password."
    echo ""
fi

# Update package lists
echo "📦 Updating package lists..."
sudo apt-get update

echo ""
echo "📦 Installing Tauri system dependencies..."
echo ""

# Install dependencies
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  build-essential \
  curl \
  wget \
  file \
  libglib2.0-dev \
  libayatana-appindicator3-dev

echo ""
echo "✅ Installation complete!"
echo ""
echo "Next steps:"
echo "  cd /home/charles/projects/Coding2025/Forge/cortex"
echo "  npm run tauri dev"
echo ""

#!/usr/bin/env bash
# Setup Windows cross-compilation using xwin (no system packages needed!)

set -e

echo "🔧 Setting up Windows cross-compilation with xwin..."
echo ""

# Install xwin if not already installed
if ! command -v xwin &> /dev/null; then
    echo "📦 Installing xwin..."
    cargo install xwin
else
    echo "✅ xwin already installed"
fi

# Download Windows SDK if not already downloaded
XWIN_DIR="$HOME/.xwin"
if [ ! -d "$XWIN_DIR" ]; then
    echo ""
    echo "📥 Downloading Windows SDK (~1GB, one-time download)..."
    echo "This may take a few minutes..."
    xwin --accept-license splat --output "$XWIN_DIR"
    echo "✅ Windows SDK downloaded to $XWIN_DIR"
else
    echo "✅ Windows SDK already downloaded"
fi

# Update .cargo/config.toml
echo ""
echo "📝 Updating .cargo/config.toml..."

cat > .cargo/config.toml << 'EOF'
# Cargo configuration for rusty_renderer

# Cross-compilation to Windows using xwin (no mingw needed!)
[target.x86_64-pc-windows-msvc]
linker = "rust-lld"
rustflags = [
    "-Lnative=$HOME/.xwin/crt/lib/x86_64",
    "-Lnative=$HOME/.xwin/sdk/lib/um/x86_64",
    "-Lnative=$HOME/.xwin/sdk/lib/ucrt/x86_64"
]

# Alternative: GNU target (requires mingw - not recommended for Bazzite)
# [target.x86_64-pc-windows-gnu]
# linker = "x86_64-w64-mingw32-gcc"
# ar = "x86_64-w64-mingw32-ar"
EOF

echo "✅ Configuration updated"

# Add Windows MSVC target if not installed
echo ""
echo "🎯 Ensuring Windows MSVC target is installed..."
rustup target add x86_64-pc-windows-msvc

echo ""
echo "✅ Setup complete!"
echo ""
echo "You can now build Windows binaries with:"
echo "  cargo build --target x86_64-pc-windows-msvc --release"
echo ""
echo "And run them with Proton using:"
echo "  ./scripts/test_dx12_proton.sh --release"
echo ""

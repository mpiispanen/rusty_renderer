#!/bin/bash
# Test DirectX in windowed mode with forward pipeline

set -e

echo "Building Windows binary..."
cargo build --release --target x86_64-pc-windows-msvc --quiet

echo "Setting up test directory..."
rm -rf windows_test_directx
mkdir -p windows_test_directx
cp target/x86_64-pc-windows-msvc/release/rusty_renderer.exe windows_test_directx/
cp -r assets scenes windows_test_directx/

cd windows_test_directx

echo ""
echo "Testing DirectX with forward pipeline (windowed mode, 60 frames)..."
echo "Press Alt+F4 or close window to stop early"
echo ""

STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$HOME/.proton_rusty_renderer" \
VKD3D_DEBUG="warn" \
"$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)/proton" run \
rusty_renderer.exe \
  --backend directx \
  --pipeline forward \
  --scene scenes/triangle.toml \
  --max-frames 60

echo ""
echo "Exit code: $?"

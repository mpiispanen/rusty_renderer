#!/bin/bash
# Simple DirectX test script

PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"
TEST_DIR="windows_test_directx"

# Copy latest binary
echo "Copying binary..."
cp target/x86_64-pc-windows-gnu/release/rusty_renderer.exe "$TEST_DIR/"

# Run triangle test
cd "$TEST_DIR"
echo "Running triangle test..."
STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$COMPAT_DATA" \
VKD3D_DEBUG="warn" \
RUST_LOG="info" \
RUST_BACKTRACE="1" \
WINEDEBUG=-all \
"$PROTON_DIR/proton" run rusty_renderer.exe \
  --backend directx \
  --scene triangle \
  --headless \
  --screenshot test_dx_triangle.png \
  --max-frames 1

if [ -f "test_dx_triangle.png" ]; then
    echo "✓ Triangle test succeeded"
    ls -lh test_dx_triangle.png
else
    echo "✗ Triangle test failed - no output"
    echo "Check rusty_renderer_debug.log for errors"
    cat rusty_renderer_debug.log
fi

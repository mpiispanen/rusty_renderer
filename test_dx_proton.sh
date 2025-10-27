#!/bin/bash
# Test DirectX backend with Proton

set -e

PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"
TEST_DIR="windows_test_directx"

echo "================================================"
echo "Testing DirectX with Proton"
echo "================================================"
echo "Proton: $PROTON_DIR"
echo "Test Directory: $TEST_DIR"
echo "================================================"
echo ""

# Change to test directory
cd "$TEST_DIR"

# Run with Proton
echo "Running rusty_renderer with DirectX backend..."
STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$COMPAT_DATA" \
VKD3D_DEBUG="warn" \
RUST_LOG="debug" \
WINEDEBUG=-all \
"$PROTON_DIR/proton" run rusty_renderer.exe --backend directx --pipeline forward --scene scenes/gltf_textured.toml --max-frames 1

EXIT_CODE=$?

echo ""
echo "================================================"
echo "Exit code: $EXIT_CODE"
echo "================================================"

# Check for output files
if [ -f "gltf_textured_dx12.png" ]; then
    echo "✓ Output image found: gltf_textured_dx12.png"
    cp gltf_textured_dx12.png ../
    echo "  Copied to project root"
fi

# Check for log file
if [ -f "rusty_renderer_debug.log" ]; then
    echo "✓ Log file found: rusty_renderer_debug.log"
    cp rusty_renderer_debug.log ../dx_debug_file.log
    echo "  Copied to project root as dx_debug_file.log"
fi

exit $EXIT_CODE

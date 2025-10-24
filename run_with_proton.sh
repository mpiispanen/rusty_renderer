#!/bin/bash
# Script to run rusty_renderer DirectX backend with Proton

# Configuration
PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"
TEST_DIR="windows_test_directx"

# Check if Proton exists
if [ ! -f "$PROTON_DIR/proton" ]; then
    echo "Error: Proton not found at $PROTON_DIR"
    echo "Available Proton versions:"
    ls -1 "$HOME/.local/share/Steam/steamapps/common/" | grep -i proton
    exit 1
fi

# Check if test directory exists
if [ ! -d "$TEST_DIR" ]; then
    echo "Error: Test directory $TEST_DIR not found"
    echo "Please build the Windows binary first:"
    echo "  cargo build --release --target x86_64-pc-windows-msvc"
    exit 1
fi

# Change to test directory
cd "$TEST_DIR"

# Default scene
SCENE="${1:-scenes/textured_cube.toml}"
VKD3D_DEBUG_LEVEL="${2:-warn}"

echo "================================================"
echo "Running rusty_renderer with Proton"
echo "================================================"
echo "Proton: $PROTON_DIR"
echo "Scene:  $SCENE"
echo "VKD3D Debug: $VKD3D_DEBUG_LEVEL"
echo "================================================"
echo ""

# Run with Proton
STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$COMPAT_DATA" \
VKD3D_DEBUG="$VKD3D_DEBUG_LEVEL" \
"$PROTON_DIR/proton" run rusty_renderer.exe --backend directx --scene "$SCENE"

EXIT_CODE=$?

echo ""
echo "================================================"
echo "Exit code: $EXIT_CODE"
echo "================================================"

exit $EXIT_CODE

#!/bin/bash
# Script to run rusty_renderer DirectX backend with Proton
# 
# Usage: ./run_with_proton.sh [OPTIONS]
#   Accepts the same options as rusty_renderer plus:
#     --vkd3d-debug <LEVEL>  VKD3D debug level (warn, info, debug) [default: warn]
#
# Examples:
#   ./run_with_proton.sh
#   ./run_with_proton.sh --scene scenes/gltf_textured_cube.toml
#   ./run_with_proton.sh --width 1920 --height 1080
#   ./run_with_proton.sh --vkd3d-debug debug

# Configuration
PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"
TEST_DIR="windows_test_directx"

# Default settings
VKD3D_DEBUG_LEVEL="warn"
APP_ARGS=()
DEFAULT_SCENE="scenes/gltf_textured.toml"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --vkd3d-debug)
            VKD3D_DEBUG_LEVEL="$2"
            shift 2
            ;;
        *)
            # Forward all other arguments to the application
            APP_ARGS+=("$1")
            shift
            ;;
    esac
done

# If no --scene argument was provided, add the default
if [[ ! " ${APP_ARGS[@]} " =~ " --scene " ]] && [[ ! " ${APP_ARGS[@]} " =~ " -s " ]]; then
    APP_ARGS+=("--scene" "$DEFAULT_SCENE")
fi

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

echo "================================================"
echo "Running rusty_renderer with Proton"
echo "================================================"
echo "Proton: $PROTON_DIR"
echo "VKD3D Debug: $VKD3D_DEBUG_LEVEL"
echo "Arguments: ${APP_ARGS[@]}"
echo "================================================"
echo ""

# Run with Proton (always use DirectX backend)
STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$COMPAT_DATA" \
VKD3D_DEBUG="$VKD3D_DEBUG_LEVEL" \
"$PROTON_DIR/proton" run rusty_renderer.exe --backend directx "${APP_ARGS[@]}"

EXIT_CODE=$?

echo ""
echo "================================================"
echo "Exit code: $EXIT_CODE"
echo "================================================"

exit $EXIT_CODE

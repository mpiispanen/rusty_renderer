#!/bin/bash
# Script to run rusty_renderer DirectX backend with Proton
# 
# Usage: ./run_with_proton.sh [OPTIONS]
#   Accepts the same options as rusty_renderer plus:
#     --vkd3d-debug <LEVEL>  VKD3D debug level (warn, info, debug) [default: warn]
#
# Debug: Check windows_test_directx/rusty_renderer_debug.log for application logs
#        Wine/Proton may not properly forward stderr/stdout
#
# Examples:
#   ./run_with_proton.sh --headless --max-frames 1
#   ./run_with_proton.sh --scene scenes/gltf_textured_cube.toml
#   ./run_with_proton.sh --width 1920 --height 1080
#   ./run_with_proton.sh --vkd3d-debug debug
#   ./run_with_proton.sh --headless --max-frames 1 --screenshot output.png

# Configuration
PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"
TEST_DIR="windows_test_directx"
BUILD_TARGETS=("x86_64-pc-windows-gnu" "x86_64-pc-windows-msvc")

# Default settings
VKD3D_DEBUG_LEVEL="warn"
APP_ARGS=()
DEFAULT_SCENE=""  # Use the app's default scene (damaged_helmet)
DEFAULT_MAX_FRAMES=""  # No frame limit by default
SCENE_PROVIDED=false
FORCE_REBUILD=false

build_windows_binary() {
    echo "  Binary missing, attempting to build a Windows target..."
    local built=false
    for target in "${BUILD_TARGETS[@]}"; do
        if [ "$target" = "x86_64-pc-windows-msvc" ]; then
            if ! command -v cargo-xwin >/dev/null 2>&1; then
                continue
            fi
            echo "    -> cargo xwin build --release --target $target"
            if cargo xwin build --release --target "$target"; then
                built=true
                break
            fi
        else
            echo "    -> cargo build --release --target $target"
            if cargo build --release --target "$target"; then
                built=true
                break
            fi
        fi
    done

    if [ "$built" = false ]; then
        echo "  ✗ Failed to compile a Windows binary automatically"
        return 1
    fi

    echo "  ✓ Windows binary built successfully"
    return 0
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --vkd3d-debug)
            VKD3D_DEBUG_LEVEL="$2"
            shift 2
            ;;
        --rebuild)
            FORCE_REBUILD=true
            shift
            ;;
        --backend|-b)
            # Skip backend argument since we always use DirectX
            shift 2
            ;;
        --scene|-s)
            SCENE_PROVIDED=true
            APP_ARGS+=("$1" "$2")
            shift 2
            ;;
        triangle|cube|gltf_textured|simple)
            # Scene name shortcut - convert to full path
            SCENE_PROVIDED=true
            APP_ARGS+=("--scene" "scenes/${1}.toml")
            shift
            ;;
        *)
            # Forward all other arguments to the application
            APP_ARGS+=("$1")
            shift
            ;;
    esac
done

# If no --scene argument was provided, add the default (if set)
if [[ "$SCENE_PROVIDED" == "false" ]] && [[ -n "$DEFAULT_SCENE" ]]; then
    APP_ARGS+=("--scene" "$DEFAULT_SCENE")
fi

# If no --max-frames argument was provided, add the default (if set)
if [[ ! " ${APP_ARGS[@]} " =~ " --max-frames " ]] && [[ -n "$DEFAULT_MAX_FRAMES" ]]; then
    APP_ARGS+=("--max-frames" "$DEFAULT_MAX_FRAMES")
fi

# Check if Proton exists
if [ ! -f "$PROTON_DIR/proton" ]; then
    echo "Error: Proton not found at $PROTON_DIR"
    echo "Available Proton versions:"
    ls -1 "$HOME/.local/share/Steam/steamapps/common/" | grep -i proton
    exit 1
fi

# Create test directory if it doesn't exist
mkdir -p "$TEST_DIR"

# Copy the latest binary and required directories automatically
echo "Syncing binary and assets..."

BINARY_COPIED=false
for TARGET in "${BUILD_TARGETS[@]}"; do
    if [ -f "target/$TARGET/release/rusty_renderer.exe" ]; then
        cp "target/$TARGET/release/rusty_renderer.exe" "$TEST_DIR/"
        echo "  ✓ Binary copied (from $TARGET)"
        BINARY_COPIED=true
        break
    fi
done

if [ "$FORCE_REBUILD" = true ]; then
    BINARY_COPIED=false
    echo "  --rebuild flag passed: rebuilding Windows binary"
    build_windows_binary || exit 1
fi

if [ "$BINARY_COPIED" = false ]; then
    if build_windows_binary; then
        for TARGET in "${BUILD_TARGETS[@]}"; do
            if [ -f "target/$TARGET/release/rusty_renderer.exe" ]; then
                cp "target/$TARGET/release/rusty_renderer.exe" "$TEST_DIR/"
                echo "  ✓ Binary copied (from $TARGET)"
                BINARY_COPIED=true
                break
            fi
        done
    fi
fi

if [ "$BINARY_COPIED" = false ]; then
    echo "  ✗ Unable to locate a Windows binary after build attempts."
    echo "    Please install the required toolchains and try again."
    exit 1
fi

# Sync shaders
if [ -d "shaders" ]; then
    mkdir -p "$TEST_DIR/shaders"
    cp -r shaders/* "$TEST_DIR/shaders/"
    echo "  ✓ Shaders synced"
fi

# Sync assets (required for textures, models, etc.)
if [ -d "assets" ]; then
    mkdir -p "$TEST_DIR/assets"
    cp -r assets/* "$TEST_DIR/assets/"
    echo "  ✓ Assets synced"
fi

# Sync scenes (required for scene files)
if [ -d "scenes" ]; then
    mkdir -p "$TEST_DIR/scenes"
    cp -r scenes/* "$TEST_DIR/scenes/"
    echo "  ✓ Scenes synced"
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
RUST_LOG="${RUST_LOG:-debug}" \
RUST_BACKTRACE="${RUST_BACKTRACE:-1}" \
WINEDEBUG=-all \
WINEDLLPATH="$PROTON_DIR/files/lib64/wine:$PROTON_DIR/files/lib64/vkd3d:$PROTON_DIR/files/lib/wine:$PROTON_DIR/files/lib/vkd3d" \
"$PROTON_DIR/proton" run rusty_renderer.exe --backend directx "${APP_ARGS[@]}"

EXIT_CODE=$?

echo ""
echo "================================================"
echo "Exit code: $EXIT_CODE"
echo "================================================"

exit $EXIT_CODE

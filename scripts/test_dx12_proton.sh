#!/bin/bash
# Test DirectX 12 backend on Linux using Proton/VKD3D translation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
BUILD_MODE="debug"
PROTON_VERSION="9.0"
ENABLE_DEBUG=false
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --release|-r)
            BUILD_MODE="release"
            shift
            ;;
        --debug|-d)
            ENABLE_DEBUG=true
            shift
            ;;
        --proton)
            PROTON_VERSION="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Test DirectX 12 backend on Linux via Proton/VKD3D"
            echo ""
            echo "Options:"
            echo "  --release, -r        Build in release mode (default: debug)"
            echo "  --debug, -d          Enable verbose debug output"
            echo "  --proton VERSION     Use specific Proton version (default: 9.0)"
            echo "                       Options: '9.0', 'experimental'"
            echo "  --dry-run            Show what would be done without executing"
            echo "  --help, -h           Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  RUST_LOG            Set Rust logging level (default: info if --debug)"
            echo "  VKD3D_DEBUG         Set VKD3D debug level (default: warn if --debug)"
            echo "  WINEDEBUG           Set Wine debug channels"
            echo ""
            echo "Examples:"
            echo "  $0                   # Build and run in debug mode"
            echo "  $0 --release         # Build and run in release mode"
            echo "  $0 --debug           # Enable all debug output"
            echo "  $0 --proton experimental  # Use Proton Experimental"
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}=== DirectX 12 on Linux via Proton ===${NC}"
echo ""

# Find Proton installation
STEAM_DIR="$HOME/.local/share/Steam"
if [ "$PROTON_VERSION" = "experimental" ]; then
    PROTON_PATH="$STEAM_DIR/steamapps/common/Proton - Experimental"
else
    PROTON_PATH="$STEAM_DIR/steamapps/common/Proton $PROTON_VERSION (Beta)"
fi

if [ ! -d "$PROTON_PATH" ]; then
    echo -e "${RED}Error: Proton not found at: $PROTON_PATH${NC}"
    echo ""
    echo "Available Proton installations:"
    ls -1 "$STEAM_DIR/steamapps/common/" | grep -i proton || echo "  None found"
    echo ""
    echo "You can specify a different version with --proton"
    exit 1
fi

echo -e "${GREEN}✓${NC} Found Proton at: $PROTON_PATH"

# Check for Windows target
if ! rustup target list --installed | grep -q "x86_64-pc-windows-gnu"; then
    echo -e "${YELLOW}⚠${NC} Windows target not installed"
    echo ""
    read -p "Install x86_64-pc-windows-gnu target? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rustup target add x86_64-pc-windows-gnu
        echo -e "${GREEN}✓${NC} Windows target installed"
    else
        echo -e "${RED}Error: Windows target required for cross-compilation${NC}"
        exit 1
    fi
fi

# Check for MinGW cross-compiler
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo -e "${YELLOW}⚠${NC} MinGW cross-compiler not found"
    echo ""
    echo "Please install it with:"
    echo "  rpm-ostree install mingw64-gcc mingw64-winpthreads-static"
    echo "  sudo systemctl reboot"
    echo ""
    echo "Or use a toolbox:"
    echo "  toolbox create crosscompile"
    echo "  toolbox enter crosscompile"
    echo "  sudo dnf install mingw64-gcc mingw64-winpthreads-static"
    exit 1
fi

echo -e "${GREEN}✓${NC} MinGW cross-compiler found"

# Build for Windows
echo ""
echo -e "${BLUE}Building for Windows ($BUILD_MODE mode)...${NC}"

BUILD_CMD="cargo build --target x86_64-pc-windows-gnu"
if [ "$BUILD_MODE" = "release" ]; then
    BUILD_CMD="$BUILD_CMD --release"
fi

if [ "$DRY_RUN" = true ]; then
    echo "Would run: $BUILD_CMD"
else
    if ! $BUILD_CMD; then
        echo -e "${RED}Error: Build failed${NC}"
        exit 1
    fi
    echo -e "${GREEN}✓${NC} Build successful"
fi

# Find the built binary
BINARY_PATH="target/x86_64-pc-windows-gnu/$BUILD_MODE/rusty_renderer.exe"

if [ ! -f "$BINARY_PATH" ] && [ "$DRY_RUN" = false ]; then
    echo -e "${RED}Error: Binary not found at: $BINARY_PATH${NC}"
    exit 1
fi

# Set up environment for Proton
export STEAM_COMPAT_CLIENT_INSTALL_PATH="$STEAM_DIR"
export STEAM_COMPAT_DATA_PATH="$HOME/.proton_rusty_renderer"

# Set up debug output if requested
if [ "$ENABLE_DEBUG" = true ]; then
    export VKD3D_DEBUG="${VKD3D_DEBUG:-warn}"
    export VKD3D_SHADER_DEBUG="${VKD3D_SHADER_DEBUG:-fixme}"
    export WINEDEBUG="${WINEDEBUG:-fixme-all,warn+d3d12,warn+dxgi}"
    export RUST_LOG="${RUST_LOG:-debug}"
    export PROTON_LOG=1
    
    echo ""
    echo -e "${YELLOW}Debug mode enabled:${NC}"
    echo "  VKD3D_DEBUG=$VKD3D_DEBUG"
    echo "  VKD3D_SHADER_DEBUG=$VKD3D_SHADER_DEBUG"
    echo "  WINEDEBUG=$WINEDEBUG"
    echo "  RUST_LOG=$RUST_LOG"
else
    # Minimal output for clean testing
    export WINEDEBUG="${WINEDEBUG:--all}"
    export RUST_LOG="${RUST_LOG:-info}"
fi

# Run via Proton
echo ""
echo -e "${BLUE}Running DirectX 12 binary via Proton...${NC}"
echo ""

PROTON_CMD="$PROTON_PATH/proton"

if [ "$DRY_RUN" = true ]; then
    echo "Would run: \"$PROTON_CMD\" run \"$BINARY_PATH\" -- --backend directx --max-frames 10"
    echo ""
    echo "Environment:"
    env | grep -E "VKD3D|WINE|RUST_LOG|PROTON" | sort
    exit 0
fi

echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
echo ""
echo "---"
echo ""

# Run the application (with proper quoting for paths with spaces)
if "$PROTON_CMD" run "$BINARY_PATH" -- --backend directx --max-frames 10; then
    echo ""
    echo "---"
    echo ""
    echo -e "${GREEN}✓${NC} Application exited successfully"
else
    EXIT_CODE=$?
    echo ""
    echo "---"
    echo ""
    echo -e "${RED}✗${NC} Application exited with code: $EXIT_CODE"
    
    if [ $EXIT_CODE -eq 1 ]; then
        echo ""
        echo "Common issues:"
        echo "  - DirectX backend not fully implemented (check rendering pipeline)"
        echo "  - VKD3D translation error (enable debug with --debug)"
        echo "  - Window creation failed (check Proton logs)"
        echo ""
        echo "Try running with --debug for more information"
    fi
    
    exit $EXIT_CODE
fi

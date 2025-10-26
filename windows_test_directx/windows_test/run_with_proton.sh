#!/bin/bash
# Script to run Windows executable with Proton/Wine

# Find Proton installation (common Steam locations)
PROTON_PATHS=(
    "$HOME/.steam/steam/steamapps/common/Proton 9.0 (Beta)/proton"
    "$HOME/.steam/steam/steamapps/common/Proton - Experimental/proton"
    "$HOME/.steam/steam/steamapps/common/Proton 8.0/proton"
    "$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)/proton"
    "$HOME/.local/share/Steam/steamapps/common/Proton - Experimental/proton"
    "$HOME/.local/share/Steam/steamapps/common/Proton 8.0/proton"
)

PROTON=""
for path in "${PROTON_PATHS[@]}"; do
    if [ -f "$path" ]; then
        PROTON="$path"
        echo "Found Proton at: $PROTON"
        break
    fi
done

# If no Proton found, try regular Wine
if [ -z "$PROTON" ]; then
    if command -v wine &> /dev/null; then
        echo "Proton not found, using Wine instead"
        WINE_CMD="wine"
    else
        echo "Error: Neither Proton nor Wine found"
        echo "Please install Wine or Steam with Proton"
        exit 1
    fi
else
    WINE_CMD="$PROTON run"
fi

# Set up environment
export STEAM_COMPAT_DATA_PATH="${STEAM_COMPAT_DATA_PATH:-$HOME/.wine_rusty_renderer}"
export STEAM_COMPAT_CLIENT_INSTALL_PATH="${STEAM_COMPAT_CLIENT_INSTALL_PATH:-$HOME/.steam/steam}"
export WINEPREFIX="${WINEPREFIX:-$HOME/.wine_rusty_renderer}"
export WINEDEBUG="${WINEDEBUG:--all}"

# Create wine prefix if needed
if [ ! -d "$WINEPREFIX" ] && [ -n "$PROTON" ]; then
    echo "Creating Wine prefix at $WINEPREFIX"
    mkdir -p "$WINEPREFIX"
fi

# Run the test with DirectX backend in headless mode
echo "Running render_graph_triangle.exe with DirectX backend..."
echo "Backend: DirectX 12"
echo "Mode: Headless (for testing)"
echo ""

cd "$(dirname "$0")"

if [ -n "$PROTON" ]; then
    "$PROTON" run ./render_graph_triangle.exe --headless directx
else
    wine ./render_graph_triangle.exe --headless directx
fi

echo ""
echo "Test complete. Check for any output images or errors above."

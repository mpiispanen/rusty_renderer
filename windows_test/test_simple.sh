#!/bin/bash
# Simple runner that captures all output

cd "$(dirname "$0")"

export STEAM_COMPAT_DATA_PATH="$HOME/.wine_rusty_renderer"
export STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.steam/steam"
export RUST_LOG=info
export WINEDEBUG=-all

echo "Running DirectX triangle test..."
"$HOME/.steam/steam/steamapps/common/Proton 9.0 (Beta)/proton" run ./render_graph_triangle.exe --headless directx > output.log 2>&1

echo "Exit code: $?"
echo "Output:"
cat output.log
echo ""
echo "Files created:"
ls -lh *.png 2>/dev/null || echo "No PNG files"

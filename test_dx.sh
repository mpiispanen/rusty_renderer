#!/bin/bash
cd windows_test_directx
STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$HOME/.proton_rusty_renderer" \
VKD3D_DEBUG="warn" \
RUST_LOG="debug" \
RUST_BACKTRACE="1" \
"$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)/proton" run rusty_renderer.exe --backend directx --scene scenes/simple_cube.toml --max-frames 1 2>&1

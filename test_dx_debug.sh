#!/bin/bash
export PROTON_DIR="/home/matpii01/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
export WINEPREFIX="/tmp/wineprefix_dx_debug"
export VKD3D_DEBUG=warn
export WINEDEBUG=-all
export RUST_LOG=debug

# Clean up old debug log
rm -f rusty_renderer_debug.log

"${PROTON_DIR}/proton" run ./target/x86_64-pc-windows-gnu/release/rusty_renderer.exe \
    --scene scenes/gltf_textured.toml \
    --max-frames 3 \
    --pipeline forward 2>&1 | tee /tmp/dx_run.log

# Show the custom debug log if it exists
if [ -f rusty_renderer_debug.log ]; then
    echo "=== Custom Debug Log ==="
    cat rusty_renderer_debug.log
fi

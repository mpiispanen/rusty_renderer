#!/bin/bash
# Test both Vulkan and DirectX backends in headless mode

set -e

echo "==========================================="
echo "Testing Vulkan Backend (headless)"
echo "==========================================="

# Build Vulkan
cargo build --release

# Test Vulkan (headless, 3 frames)
rm -f gltf_textured_vulkan.png
cargo run --release -- --headless --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward --max-frames 3 --screenshot gltf_textured_vulkan.png

# Check output
if [ -f "gltf_textured_vulkan.png" ]; then
    echo "✓ Vulkan output image created"
    ls -lh gltf_textured_vulkan.png
else
    echo "✗ Vulkan output image NOT created"
fi

echo ""
echo "==========================================="
echo "Testing DirectX Backend (headless, Proton)"
echo "==========================================="

# Build DirectX
cargo build --target x86_64-pc-windows-msvc --release
cp target/x86_64-pc-windows-msvc/release/rusty_renderer.exe windows_test_directx/gltf_viewer.exe

# Test DirectX
cd windows_test_directx
rm -f gltf_textured_dx12.png

PROTON_DIR="$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)"
COMPAT_DATA="$HOME/.proton_rusty_renderer"

STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$COMPAT_DATA" \
VKD3D_DEBUG="warn" \
RUST_LOG="info" \
"$PROTON_DIR/proton" run gltf_viewer.exe --headless --backend directx --scene scenes/gltf_textured.toml --pipeline forward --max-frames 3 --screenshot gltf_textured_dx12.png

# Check output
if [ -f "gltf_textured_dx12.png" ]; then
    echo "✓ DirectX output image created"
    cp gltf_textured_dx12.png ../
    ls -lh gltf_textured_dx12.png
else
    echo "✗ DirectX output image NOT created"
fi

cd ..

echo ""
echo "==========================================="
echo "Test Complete - Comparing Images"
echo "==========================================="
ls -lh gltf_textured_*.png 2>/dev/null || echo "No output images"

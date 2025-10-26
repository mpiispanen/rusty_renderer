#!/bin/bash
# Test rendering parity between Vulkan and DirectX backends

echo "Testing Vulkan backend..."
RUST_LOG=warn RENDERER_BACKEND=vulkan timeout --signal=KILL 10 cargo run --release -- --max-frames 1 --scene scenes/gltf_textured.toml --screenshot vulkan_test.png 2>&1 | tail -5

echo ""
echo "Testing DirectX backend with Proton..."
./run_with_proton.sh --max-frames 1 --screenshot directx_test.png 2>&1 | grep "Exit code"

echo ""
echo "Screenshots:"
ls -lh vulkan_test.png directx_test.png 2>/dev/null || echo "One or more screenshots not generated"

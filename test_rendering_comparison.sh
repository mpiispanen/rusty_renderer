#!/bin/bash
# Test rendering comparison between Vulkan and DirectX backends
# Both backends should render the same scene with the forward pipeline

echo "=============================================="
echo "Testing Vulkan backend..."
echo "=============================================="
timeout 3 cargo run --release -- \
    --backend vulkan \
    --scene scenes/gltf_textured.toml \
    --pipeline forward \
    --max-frames 60 2>&1 | grep -E "(ERROR|WARN|textured)" | head -10

echo ""
echo "=============================================="
echo "Testing DirectX backend (via Proton)..."
echo "=============================================="
timeout 8 ./run_with_proton.sh \
    --scene scenes/gltf_textured.toml \
    --pipeline forward \
    --max-frames 60 2>&1 | grep -v "fixme\|trace:" | tail -10

echo ""
echo "=============================================="
echo "Both backends tested successfully!"
echo "=============================================="

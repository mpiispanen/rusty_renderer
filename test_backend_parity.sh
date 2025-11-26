#!/bin/bash
# Test backend parity between Vulkan and DirectX
# Renders the same scene with both backends and saves screenshots

set -e

SCENE="${1:-scenes/gltf_textured.toml}"
PIPELINE="${2:-forward}"
MAX_FRAMES=5

echo "==================================="
echo "Backend Parity Test"
echo "==================================="
echo "Scene: $SCENE"
echo "Pipeline: $PIPELINE"
echo "Max frames: $MAX_FRAMES"
echo ""

# Test Vulkan
echo "Testing Vulkan backend..."
BACKEND=vulkan cargo run --release -- \
    --scene "$SCENE" \
    --max-frames "$MAX_FRAMES" \
    --headless \
    --screenshot test.png \
    2>&1 | grep -E "INFO|ERROR|WARN" | tail -20

if [ -f test.png ]; then
    mv test.png vulkan_output.png
    echo "✓ Vulkan output saved to vulkan_output.png"
else
    echo "✗ Vulkan did not produce output image"
fi

echo ""

# Test DirectX (via Proton)
echo "Testing DirectX backend (via Proton)..."
./run_with_proton.sh \
    --scene "$SCENE" \
    --max-frames "$MAX_FRAMES" \
    --headless \
    --screenshot test.png \
    2>&1 | grep -E "INFO|ERROR|WARN" | grep -v "vkd3d-proton" | tail -20

if [ -f windows_test_directx/test.png ]; then
    cp windows_test_directx/test.png directx_output.png
    echo "✓ DirectX output saved to directx_output.png"
else
    echo "✗ DirectX did not produce output image"
fi

echo ""
echo "==================================="
echo "Comparison"
echo "==================================="

if [ -f vulkan_output.png ] && [ -f directx_output.png ]; then
    echo "Both outputs generated successfully!"
    echo ""
    echo "Files:"
    ls -lh vulkan_output.png directx_output.png
    echo ""
    echo "You can compare them visually or use imagemagick:"
    echo "  compare vulkan_output.png directx_output.png diff.png"
else
    echo "One or both backends failed to generate output"
    exit 1
fi

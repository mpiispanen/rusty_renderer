#!/bin/bash
# Compare Vulkan and DirectX rendering outputs

set -e

echo "================================================"
echo "Backend Comparison Test"
echo "================================================"
echo ""

# Test scene
SCENE="scenes/gltf_textured.toml"

# Test Vulkan
echo "Testing Vulkan backend..."
./target/release/examples/gltf_viewer vulkan "$SCENE" 2>&1 | grep -E "(✓|Error)" || true
echo ""

# Test DirectX via Proton
echo "Testing DirectX backend via Proton..."
./test_dx_proton.sh 2>&1 | grep -E "(✓|Error|Exit code)" || true
echo ""

# Check outputs
echo "================================================"
echo "Output Comparison"
echo "================================================"

if [ -f "gltf_textured_vulkan.png" ] && [ -f "gltf_textured_dx12.png" ]; then
    VK_SIZE=$(stat -c%s "gltf_textured_vulkan.png")
    DX_SIZE=$(stat -c%s "gltf_textured_dx12.png")
    
    echo "✓ Both backends produced output"
    echo "  Vulkan: $VK_SIZE bytes"
    echo "  DirectX12: $DX_SIZE bytes"
    echo ""
    
    # Check if images are valid PNGs
    if file gltf_textured_vulkan.png | grep -q "PNG image"; then
        VK_DIM=$(identify -format "%wx%h" gltf_textured_vulkan.png 2>/dev/null || echo "unknown")
        echo "  Vulkan dimensions: $VK_DIM"
    fi
    
    if file gltf_textured_dx12.png | grep -q "PNG image"; then
        DX_DIM=$(identify -format "%wx%h" gltf_textured_dx12.png 2>/dev/null || echo "unknown")
        echo "  DirectX12 dimensions: $DX_DIM"
    fi
    
    echo ""
    echo "✓ Backend comparison complete"
    echo ""
    echo "To view outputs:"
    echo "  eog gltf_textured_vulkan.png gltf_textured_dx12.png"
else
    echo "✗ Missing output files"
    exit 1
fi

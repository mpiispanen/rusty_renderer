#!/bin/bash
# Test both rendering backends after winding order fix

set -e

echo "==================================="
echo "Testing Vulkan Backend"
echo "==================================="
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --max-frames 1 --headless --screenshot vulkan_winding_fixed.png 2>&1 | tail -5
echo ""
echo "Vulkan screenshot saved: vulkan_winding_fixed.png"
echo ""

echo "==================================="
echo "Testing DirectX Backend (via Proton)"
echo "==================================="
./run_with_proton.sh --max-frames 1 --headless --screenshot dx_winding_fixed.png 2>&1 | tail -10
echo ""
echo "DirectX screenshot saved: windows_test_directx/dx_winding_fixed.png"
echo ""

echo "==================================="
echo "Tests complete!"
echo "==================================="
echo "Compare screenshots:"
echo "  - vulkan_winding_fixed.png"
echo "  - windows_test_directx/dx_winding_fixed.png"

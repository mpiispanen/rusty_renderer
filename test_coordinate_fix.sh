#!/bin/bash

echo "Testing coordinate system fix..."
echo ""
echo "1. Testing Vulkan backend..."
timeout 5 cargo run --release -- --scene scenes/gltf_textured.toml --pipeline forward 2>&1 | grep -E "(INFO|WARN|ERROR)" | tail -20

echo ""
echo "2. Testing DirectX backend with Proton..."
./run_with_proton.sh 2>&1 | grep -v "^wine:" | grep -v "vkd3d-proton" | tail -20

echo ""
echo "Test complete. Please check if both renderers show the cube oriented the same way."

#!/bin/bash

echo "================================================"
echo "Testing Vulkan and DirectX coordinate systems"
echo "================================================"
echo ""
echo "Instructions:"
echo "1. First window will show Vulkan rendering"
echo "2. Second window will show DirectX rendering"  
echo "3. Compare if the cube orientation matches"
echo ""
read -p "Press Enter to start Vulkan test..."

echo "Starting Vulkan..."
timeout 5 cargo run --release -- --scene scenes/gltf_textured.toml --pipeline forward 2>&1 | tail -5

echo ""
read -p "Press Enter to start DirectX test..."

echo "Starting DirectX..."
./run_with_proton.sh

echo ""
echo "Test complete!"

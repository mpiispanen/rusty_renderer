#!/bin/bash

echo "Testing Vulkan backend..."
timeout 5 cargo run --release -- --scene scenes/gltf_textured.toml &
sleep 4
killall -9 rusty_renderer 2>/dev/null

echo ""
echo "Testing DirectX backend via Proton..."
cd windows_test_directx && timeout 5 ../run_with_proton.sh ./gltf_viewer.exe &
sleep 4
killall -9 gltf_viewer.exe 2>/dev/null

echo ""
echo "Both tests complete"

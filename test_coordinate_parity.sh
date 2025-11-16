#!/bin/bash
# Test coordinate system parity between Vulkan and DirectX backends

set -e

echo "================================"
echo "Testing Coordinate System Parity"
echo "================================"

# Build first
echo "Building..."
cargo build --release 2>&1 | tail -5

echo ""
echo "Test 1: Cube Scene"
echo "-------------------"
echo "Testing Vulkan (cube)..."
./target/release/rusty_renderer --backend vulkan --headless \
    --screenshot /var/home/matpii01/test_vk_cube.png \
    --scene scenes/cube.toml 2>&1 | grep -E "Scene loaded|Screenshot saved"

echo "Testing DirectX (cube)..."
./run_with_proton.sh --headless \
    --screenshot /var/home/matpii01/test_dx_cube.png \
    --scene scenes/cube.toml 2>&1 | tail -3

echo ""
echo "Test 2: Damaged Helmet"
echo "----------------------"
echo "Testing Vulkan (helmet)..."
./target/release/rusty_renderer --backend vulkan --headless \
    --screenshot /var/home/matpii01/test_vk_helmet.png \
    --scene scenes/damaged_helmet.toml 2>&1 | grep -E "Scene loaded|Screenshot saved"

echo "Testing DirectX (helmet)..."
./run_with_proton.sh --headless \
    --screenshot /var/home/matpii01/test_dx_helmet.png \
    --scene scenes/damaged_helmet.toml 2>&1 | tail -3

echo ""
echo "Creating comparison images..."
magick /var/home/matpii01/test_vk_cube.png /var/home/matpii01/test_dx_cube.png \
    +append /var/home/matpii01/test_cube_comparison.png
echo "  ✓ test_cube_comparison.png (VK | DX)"

magick /var/home/matpii01/test_vk_helmet.png /var/home/matpii01/test_dx_helmet.png \
    +append /var/home/matpii01/test_helmet_comparison.png
echo "  ✓ test_helmet_comparison.png (VK | DX)"

echo ""
echo "================================"
echo "Testing Complete!"
echo "================================"
echo "Review the comparison images:"
echo "  - test_cube_comparison.png"
echo "  - test_helmet_comparison.png"
echo ""
echo "Both backends should show identical orientation."

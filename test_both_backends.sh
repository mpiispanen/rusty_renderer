#!/bin/bash
# Quick test script to verify both backends render correctly

set -e

echo "=========================================="
echo "Testing Coordinate System Fix"
echo "=========================================="
echo ""

echo "Building projects..."
echo "  - Linux (Vulkan) build..."
cargo build --release --quiet 2>&1 | tail -3

echo "  - Windows (DirectX) build..."
cargo build --release --target x86_64-pc-windows-msvc --quiet 2>&1 | tail -3

echo ""
echo "✓ Builds complete"
echo ""
echo "=========================================="
echo "Manual Test Required:"
echo "=========================================="
echo ""
echo "1. Test Vulkan:"
echo "   cargo run --release -- --scene scenes/gltf_textured.toml --pipeline forward"
echo ""
echo "2. Test DirectX:"
echo "   ./run_with_proton.sh"
echo ""
echo "3. Verify both show the cube with the same orientation"
echo "   - The checkerboard texture should be visible"
echo "   - The cube should not be upside down or mirrored"
echo ""

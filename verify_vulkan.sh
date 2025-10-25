#!/bin/bash
# Verify Vulkan backend works completely

set -e

echo "========================================="
echo "Vulkan Backend Verification"
echo "========================================="
echo

# Clean up old screenshots
rm -f vk_*.png

SCENES=(
    "scenes/triangle.toml"
    "scenes/textured_cube.toml"
    "scenes/gltf_textured.toml"
)

for scene in "${SCENES[@]}"; do
    name=$(basename "$scene" .toml)
    echo -n "Testing $name... "
    
    if cargo run --release -- \
        --backend vulkan \
        --pipeline forward \
        --scene "$scene" \
        --headless \
        --screenshot "vk_${name}.png" \
        --max-frames 1 \
        > /dev/null 2>&1; then
        
        if [ -f "vk_${name}.png" ]; then
            size=$(stat -c%s "vk_${name}.png")
            if [ "$size" -gt 1000 ]; then
                echo "✅ ($size bytes)"
            else
                echo "❌ (file too small: $size bytes)"
            fi
        else
            echo "❌ (no screenshot)"
        fi
    else
        echo "❌ (failed)"
    fi
done

echo
echo "========================================="
echo "Screenshots generated:"
ls -lh vk_*.png 2>/dev/null | awk '{print $9, $5}'
echo "========================================="

#!/bin/bash
# Script to update golden reference images
# Usage: ./scripts/update_golden_references.sh [--vulkan|--directx|--all]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REF_DIR="$REPO_ROOT/references/gltf_textured"

# Parse arguments
UPDATE_MODE="${1:---all}"

echo "🎨 Golden Reference Updater"
echo "==========================="
echo ""

# Create reference directory if it doesn't exist
mkdir -p "$REF_DIR"

update_vulkan() {
    echo "📸 Updating Vulkan golden reference..."
    
    # Render with Vulkan
    mkdir -p "$REPO_ROOT/screenshots/temp"
    cargo run --release -- \
        --scene scenes/gltf_textured.toml \
        --backend vulkan \
        --pipeline forward \
        --headless \
        --screenshot "$REPO_ROOT/screenshots/temp/vulkan_temp.png"
    
    # Copy to golden reference
    cp "$REPO_ROOT/screenshots/temp/vulkan_temp.png" "$REF_DIR/gltf_textured_vulkan.png"
    echo "✅ Vulkan golden reference updated: $REF_DIR/gltf_textured_vulkan.png"
    
    # Show file info
    ls -lh "$REF_DIR/gltf_textured_vulkan.png"
}

update_directx() {
    echo "📸 Updating DirectX golden reference..."
    echo "⚠️  DirectX requires Windows - skipping on Linux"
    echo "   Run this on Windows machine or let Windows CI create it"
}

case "$UPDATE_MODE" in
    --vulkan)
        update_vulkan
        ;;
    --directx)
        update_directx
        ;;
    --all)
        update_vulkan
        update_directx
        ;;
    *)
        echo "Usage: $0 [--vulkan|--directx|--all]"
        exit 1
        ;;
esac

# Cleanup temp directory
rm -rf "$REPO_ROOT/screenshots/temp"

echo ""
echo "✅ Golden references updated!"
echo ""
echo "Next steps:"
echo "1. Review the new reference images visually"
echo "2. If they look correct, commit them:"
echo "   git add references/gltf_textured/"
echo "   git commit -m 'Update golden references for gltf_textured scene'"
echo "3. CI will now compare against these references"

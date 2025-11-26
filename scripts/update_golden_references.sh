#!/bin/bash
# Script to update golden reference images
# Usage: ./scripts/update_golden_references.sh [--vulkan|--directx|--all]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REF_DIR="$REPO_ROOT/references/damaged_helmet"

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
        --scene scenes/damaged_helmet.toml \
        --backend vulkan \
        --headless \
        --max-frames 1 \
        --screenshot "$REPO_ROOT/screenshots/temp/vulkan_temp.png"
    
    # Copy to golden reference
    cp "$REPO_ROOT/screenshots/temp/vulkan_temp.png" "$REF_DIR/damaged_helmet_vulkan.png"
    echo "✅ Vulkan golden reference updated: $REF_DIR/damaged_helmet_vulkan.png"
    
    # Show file info
    ls -lh "$REF_DIR/damaged_helmet_vulkan.png"
}

update_directx() {
    echo "📸 Updating DirectX golden reference..."
    
    if [ -f "$REPO_ROOT/run_with_proton.sh" ]; then
        # Use Proton script
        mkdir -p "$REPO_ROOT/screenshots/temp"
        "$REPO_ROOT/run_with_proton.sh" \
            --scene scenes/damaged_helmet.toml \
            --headless \
            --max-frames 1 \
            --screenshot "$REPO_ROOT/screenshots/temp/directx_temp.png"
        
        # The screenshot might be in windows_test_directx/screenshots/temp/directx_temp.png depending on how run_with_proton works
        # run_with_proton.sh runs from windows_test_directx, so relative paths are relative to that.
        # But wait, run_with_proton.sh takes arguments and passes them to the app.
        # The app resolves paths relative to CWD.
        # run_with_proton.sh changes directory to windows_test_directx.
        
        # Let's check where the file ends up.
        # If we pass absolute path to screenshot, it might work if mapped correctly, but safer to use relative and find it.
        
        # Actually, let's just use a simple filename and find it.
        "$REPO_ROOT/run_with_proton.sh" \
            --scene scenes/damaged_helmet.toml \
            --headless \
            --max-frames 1 \
            --screenshot "directx_ref.png"
            
        SRC_FILE="$REPO_ROOT/windows_test_directx/directx_ref.png"
        
        if [ -f "$SRC_FILE" ]; then
            cp "$SRC_FILE" "$REF_DIR/damaged_helmet_directx.png"
            echo "✅ DirectX golden reference updated: $REF_DIR/damaged_helmet_directx.png"
            ls -lh "$REF_DIR/damaged_helmet_directx.png"
        else
            echo "❌ Failed to generate DirectX screenshot"
        fi
    else
        echo "⚠️  DirectX update skipped (run_with_proton.sh not found)"
    fi
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

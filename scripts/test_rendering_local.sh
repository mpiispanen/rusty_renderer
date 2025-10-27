#!/bin/bash
# Test rendering locally before pushing to CI

set -e

echo "=== Local Rendering Test ==="
echo ""

# Build in release mode
echo "📦 Building release binary..."
cargo build --release --quiet

# Create screenshot directory
mkdir -p screenshots/local/{vulkan,directx}

# Test Vulkan rendering
echo ""
echo "🎨 Testing Vulkan rendering..."
RUST_LOG=warn ./target/release/rusty_renderer \
  --scene scenes/gltf_textured.toml \
  --backend vulkan \
  --pipeline forward \
  --headless \
  --screenshot screenshots/local/vulkan/gltf_textured.png

if [ -f "screenshots/local/vulkan/gltf_textured.png" ]; then
  size=$(stat -f%z "screenshots/local/vulkan/gltf_textured.png" 2>/dev/null || stat -c%s "screenshots/local/vulkan/gltf_textured.png")
  echo "✅ Vulkan screenshot created ($size bytes)"
else
  echo "❌ Vulkan screenshot failed"
  exit 1
fi

# Test DirectX rendering (if on Windows or with Wine/Proton)
if command -v wine64 &> /dev/null || [ -f "run_with_proton.sh" ]; then
  echo ""
  echo "🎨 Testing DirectX rendering..."
  
  if [ -f "run_with_proton.sh" ]; then
    # Use Proton script - note that it runs from windows_test_directx directory
    ./run_with_proton.sh --scene scenes/gltf_textured.toml --headless --screenshot screenshots/local/directx/gltf_textured.png
    
    # Copy screenshot from windows_test_directx to main screenshots directory
    if [ -f "windows_test_directx/screenshots/local/directx/gltf_textured.png" ]; then
      mkdir -p screenshots/local/directx
      cp windows_test_directx/screenshots/local/directx/gltf_textured.png screenshots/local/directx/
    fi
  else
    # Try Wine
    wine64 ./target/x86_64-pc-windows-gnu/release/rusty_renderer.exe \
      --scene scenes/gltf_textured.toml \
      --backend directx \
      --pipeline forward \
      --headless \
      --screenshot screenshots/local/directx/gltf_textured.png 2>/dev/null || {
        echo "⚠️  DirectX test skipped (Wine/Proton not available)"
      }
  fi
  
  if [ -f "screenshots/local/directx/gltf_textured.png" ]; then
    size=$(stat -f%z "screenshots/local/directx/gltf_textured.png" 2>/dev/null || stat -c%s "screenshots/local/directx/gltf_textured.png")
    echo "✅ DirectX screenshot created ($size bytes)"
  fi
else
  echo ""
  echo "ℹ️  DirectX test skipped (not on Windows, Wine/Proton not available)"
fi

# Compare backends if both exist
echo ""
if [ -f "screenshots/local/vulkan/gltf_textured.png" ] && [ -f "screenshots/local/directx/gltf_textured.png" ]; then
  echo "🔍 Comparing backend outputs..."
  
  if command -v python3 &> /dev/null && python3 -c "import flip_evaluator" 2>/dev/null; then
    mkdir -p screenshots/local/comparisons
    
    python3 scripts/flip_compare.py \
      screenshots/local/vulkan/gltf_textured.png \
      screenshots/local/directx/gltf_textured.png \
      screenshots/local/comparisons/vulkan_vs_directx.png \
      --threshold 0.05 && {
        echo "✅ Backend parity check PASSED"
      } || {
        echo "⚠️  Backends differ - check screenshots/local/comparisons/"
      }
  else
    echo "ℹ️  FLIP comparison not available (install: pip install flip-evaluator)"
  fi
fi

# Show generated screenshots
echo ""
echo "=== Generated Screenshots ==="
find screenshots/local -name "*.png" -exec ls -lh {} \;

echo ""
echo "✅ Local rendering test complete"
echo "   Screenshots saved to: screenshots/local/"

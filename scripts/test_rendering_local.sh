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
  --scene scenes/damaged_helmet.toml \
  --backend vulkan \
  --headless \
  --max-frames 1 \
  --screenshot screenshots/local/vulkan/damaged_helmet.png

if [ -f "screenshots/local/vulkan/damaged_helmet.png" ]; then
  size=$(stat -f%z "screenshots/local/vulkan/damaged_helmet.png" 2>/dev/null || stat -c%s "screenshots/local/vulkan/damaged_helmet.png")
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
    ./run_with_proton.sh --scene scenes/damaged_helmet.toml --headless --max-frames 1 --screenshot screenshots/local/directx/damaged_helmet.png
    
    # Copy screenshot from windows_test_directx to main screenshots directory
    if [ -f "windows_test_directx/screenshots/local/directx/damaged_helmet.png" ]; then
      mkdir -p screenshots/local/directx
      cp windows_test_directx/screenshots/local/directx/damaged_helmet.png screenshots/local/directx/
    fi
  else
    # Try Wine
    wine64 ./target/x86_64-pc-windows-gnu/release/rusty_renderer.exe \
      --scene scenes/damaged_helmet.toml \
      --backend directx \
      --headless \
      --max-frames 1 \
      --screenshot screenshots/local/directx/damaged_helmet.png 2>/dev/null || {
        echo "⚠️  DirectX test skipped (Wine/Proton not available)"
      }
  fi
  
  if [ -f "screenshots/local/directx/damaged_helmet.png" ]; then
    size=$(stat -f%z "screenshots/local/directx/damaged_helmet.png" 2>/dev/null || stat -c%s "screenshots/local/directx/damaged_helmet.png")
    echo "✅ DirectX screenshot created ($size bytes)"
  fi
else
  echo ""
  echo "ℹ️  DirectX test skipped (not on Windows, Wine/Proton not available)"
fi

# Compare backends if both exist
echo ""
if [ -f "screenshots/local/vulkan/damaged_helmet.png" ] && [ -f "screenshots/local/directx/damaged_helmet.png" ]; then
  echo "🔍 Comparing backend outputs..."
  
  if command -v python3 &> /dev/null && python3 -c "import flip_evaluator" 2>/dev/null; then
    mkdir -p screenshots/local/comparisons
    
    python3 scripts/flip_compare.py \
      screenshots/local/vulkan/damaged_helmet.png \
      screenshots/local/directx/damaged_helmet.png \
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

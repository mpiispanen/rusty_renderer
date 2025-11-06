#!/bin/bash
# Test script to capture debug info about the rendering artifact

cd ~/rusty_renderer

# Clean old logs
rm -f rusty_renderer_debug.log matrix_debug.log

# Run headless to get a screenshot
echo "Running headless mode to capture state..."
timeout 15 cargo run --release -- --headless --scene damaged_helmet --screenshot screenshots/artifact_test.png --max-frames 1 2>&1 | tee artifact_test.log

echo ""
echo "=== Last 30 lines of debug output ==="
tail -30 artifact_test.log

echo ""
echo "=== Matrix debug (if available) ==="
if [ -f matrix_debug.log ]; then
    tail -20 matrix_debug.log
fi

echo ""
echo "=== Check for multiple draw calls ==="
grep -i "drawing\|draw_indexed" artifact_test.log

echo ""
echo "Screenshot saved to: screenshots/artifact_test.png"
ls -lh screenshots/artifact_test.png 2>/dev/null || echo "Screenshot not found"

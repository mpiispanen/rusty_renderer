#!/bin/bash
# Interactive camera test
# Run this to test camera movement controls
# 
# Controls:
#   WASD - Move forward/backward/left/right
#   Q/E - Move down/up
#   Mouse - Look around (when window is focused)
#   Shift - Move faster
#   ESC - Exit

echo "=== Interactive Camera Test ==="
echo ""
echo "This will launch the renderer with a free-fly camera."
echo "Use WASD to move, Q/E for up/down, mouse to look around."
echo "Hold Shift to move faster. Press ESC to exit."
echo ""
echo "Starting in 3 seconds..."
sleep 3

cargo run --release -- \
    --backend vulkan \
    --scene camera_test \
    --width 1280 \
    --height 720

echo ""
echo "Interactive camera test complete!"

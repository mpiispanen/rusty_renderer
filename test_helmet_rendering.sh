#!/bin/bash
echo "Testing helmet rendering with textures..."
cargo run --release -- --scene damaged_helmet &
APP_PID=$!
sleep 8
kill $APP_PID 2>/dev/null
wait $APP_PID 2>/dev/null
echo "Test complete!"

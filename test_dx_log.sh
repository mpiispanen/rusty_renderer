#!/bin/bash
rm -f rusty_renderer_debug.log
export RUST_LOG=debug
export WINEDEBUG=-all
./run_with_proton.sh 2>&1 | tee dx_run.log
echo "=== Debug log contents ==="
cat rusty_renderer_debug.log 2>/dev/null || echo "No debug log found"

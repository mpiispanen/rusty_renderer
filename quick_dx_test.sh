#!/bin/bash
# Quick test script to verify DX12 shader is working

cd /var/home/matpii01/rusty_renderer

echo "Building Windows binary..."
cargo build --release --target x86_64-pc-windows-gnu 2>&1 | tail -3

echo ""
echo "Running DX12 test (3 seconds)..."
timeout 3 ./run_with_proton.sh 2>&1 | grep -E "(error|Error|ERROR)" || echo "No errors detected"

echo ""
echo "Test complete!"

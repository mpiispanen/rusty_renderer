#!/bin/bash
# Quick DirectX test script - runs headless mode by default

echo "Building DirectX backend..."
BACKEND=directx cargo build --release --target x86_64-pc-windows-gnu -q

if [ $? -ne 0 ]; then
    echo "Build failed!"
    exit 1
fi

echo "Copying binary..."
cp target/x86_64-pc-windows-gnu/release/rusty_renderer.exe windows_test_directx/

echo "Running with Proton (headless mode)..."
./run_with_proton.sh --max-frames 3 --headless --scene scenes/gltf_textured.toml --pipeline forward "$@"

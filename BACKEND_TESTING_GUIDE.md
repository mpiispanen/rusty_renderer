# Quick Backend Testing Guide

**Last Updated:** 2025-10-25

## Run Each Backend

### Vulkan (Native Linux/Windows)
```bash
# Production ready, zero validation errors
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward --max-frames 60
```

### DirectX 12 (via Proton on Linux)
```bash
# Use the helper script
./run_with_proton.sh --max-frames 60

# Or manually
cd windows_test_directx
WINEPREFIX=~/.proton \
VKD3D_CONFIG=dxr11,dxr \
PROTON_LOG=1 \
~/.steam/root/compatibilitytools.d/GE-Proton9-16/files/bin/wine64 \
./rusty_renderer.exe --backend directx --scene scenes/gltf_textured.toml --pipeline forward --max-frames 60
```

### wgpu (Cross-platform)
```bash
# ⚠️ Must use windowed mode (headless hangs)
# ⚠️ Must use Forward pipeline (Simple pipeline has no bind groups)
cargo run --release -- --backend wgpu --scene scenes/gltf_textured.toml --pipeline forward --max-frames 60
```

## Common Options

### All Backends
```bash
--backend <vulkan|directx|wgpu>     # Choose backend
--scene <path>                       # Scene file (TOML)
--pipeline <simple|forward>          # Rendering pipeline
--max-frames <n>                     # Exit after N frames
--headless                           # No window (Vulkan/DirectX only)
```

### Available Scenes
```bash
# List all available scenes
cargo run -- --list-scenes

# Common scenes:
scenes/triangle.toml          # Simple RGB triangle
scenes/gltf_textured.toml    # Textured cube with lighting
scenes/cube.toml             # Basic lit cube
```

## Backend Comparison Test

Run the same scene on all backends:

```bash
# Vulkan
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward --max-frames 5

# DirectX (via Proton)
./run_with_proton.sh --max-frames 5

# wgpu
cargo run --release -- --backend wgpu --scene scenes/gltf_textured.toml --pipeline forward --max-frames 5
```

## Current Limitations

### Vulkan ✅
- **Status:** Production ready
- **Platforms:** Linux, Windows
- **Modes:** Windowed, Headless
- **Pipelines:** Simple, Forward
- **Notes:** Zero validation errors

### DirectX 12 ✅
- **Status:** Working
- **Platforms:** Windows native, Linux via Proton
- **Modes:** Windowed, Headless (via Proton)
- **Pipelines:** Forward (Simple has issues)
- **Notes:** 
  - No depth testing yet (#71)
  - No texture support yet (needs descriptor tables)
  - Use `forward_simple.hlsl` shader

### wgpu ✅
- **Status:** Working (with limitations)
- **Platforms:** Linux, Windows, macOS, Web (future)
- **Modes:** Windowed only
- **Pipelines:** Forward only
- **Notes:**
  - ❌ Headless mode hangs (known wgpu issue)
  - ❌ Simple pipeline incompatible (no bind groups)
  - ✅ Use for macOS development or cross-platform testing

## Troubleshooting

### Vulkan
```bash
# Enable validation layers
cargo run -- --backend vulkan --enable-validation

# Check Vulkan drivers
vulkaninfo | head -20
```

### DirectX (Proton)
```bash
# Check Proton logs
tail -f windows_test_directx/rusty_renderer_debug.log

# Test if Proton works
./run_with_proton.sh --max-frames 1
```

### wgpu
```bash
# wgpu only supports windowed mode currently
# Don't use --headless flag

# Must use Forward pipeline
cargo run --release -- --backend wgpu --scene scenes/gltf_textured.toml --pipeline forward

# NOT this (will fail):
# cargo run -- --backend wgpu --scene scenes/triangle.toml  # ❌ Simple pipeline
```

## Screenshot/Headless Rendering

### Vulkan (Headless)
```bash
cargo run --release -- \
  --backend vulkan \
  --scene scenes/gltf_textured.toml \
  --pipeline forward \
  --headless \
  --max-frames 1
```

### DirectX (Headless via Proton)
```bash
# Already headless by default in Proton
./run_with_proton.sh --max-frames 1
```

### wgpu
```bash
# Headless not currently working - use windowed mode
# Window will open but you can close it after rendering
```

## Build for Specific Backend

If you only want to test one backend:

```bash
# Vulkan only
cargo run --release --no-default-features --features vulkan -- --backend vulkan

# Note: Currently all backends are built by default
# Feature flags may be added in Phase 2
```

## Performance Testing

```bash
# Vulkan (fastest)
time cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward --max-frames 1000 --headless

# DirectX (via Proton translation layer)
time ./run_with_proton.sh --max-frames 1000

# wgpu (additional abstraction layer)
time cargo run --release -- --backend wgpu --scene scenes/gltf_textured.toml --pipeline forward --max-frames 1000
```

## Next Steps

See [ARCHITECTURE_REFACTOR_STATUS.md](./ARCHITECTURE_REFACTOR_STATUS.md) for:
- Phase 1: Backend Parity (DirectX depth testing, visual consistency)
- CI/CD: Automated comparison testing
- GitHub Issues: [#71](https://github.com/mpiispanen/rusty_renderer/issues/71), [#72](https://github.com/mpiispanen/rusty_renderer/issues/72), [#74](https://github.com/mpiispanen/rusty_renderer/issues/74)

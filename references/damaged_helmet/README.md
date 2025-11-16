# Damaged Helmet Reference Images

**Scene:** `scenes/damaged_helmet.toml`  
**Created:** 2025-11-16  
**Resolution:** 1280x720  
**Model:** glTF DamagedHelmet.glb from Khronos glTF-Sample-Models  
**Description:** PBR-textured helmet model with complex materials

## Purpose

This is the **default scene** for visual regression testing. It tests:
- glTF model loading
- PBR material rendering
- Texture mapping
- Forward rendering pass
- Complex geometry

## Current Baselines

| Backend | Image | Status | Notes |
|---------|-------|--------|-------|
| Vulkan | damaged_helmet_vulkan.png | ✅ Available | Generated 2025-11-16 |
| DirectX | damaged_helmet_directx.png | ⏳ Pending | To be added when DX backend is stable |

## Expected Visual Features

- Metallic helmet with rust/damage texture
- Correct PBR shading
- Proper texture mapping
- No black/missing textures
- Camera position: (0, 0, 3) looking at origin

## Known Issues

- DirectX backend currently has synchronization issues preventing headless rendering
- Once DX rendering is fixed, golden reference will be added

## Updating

To update the Vulkan reference:

```bash
cargo build --release
./target/release/rusty_renderer \
  --scene scenes/damaged_helmet.toml \
  --backend vulkan \
  --headless \
  --max-frames 1 \
  --screenshot references/damaged_helmet/damaged_helmet_vulkan.png
```

To add DirectX reference (when stable):

```bash
# On Windows or with Wine/Proton
./target/release/rusty_renderer.exe \
  --scene scenes/damaged_helmet.toml \
  --backend directx \
  --headless \
  --max-frames 1 \
  --screenshot references/damaged_helmet/damaged_helmet_directx.png
```

## Expected FLIP Errors

Once both backends are working correctly:
- Vulkan vs DirectX: < 0.10 (small differences expected due to driver/platform)

# Backend Rendering Comparison

## Current Status (2025-10-27)

### Vulkan Backend
- ✅ Cube renders with correct texturing
- ✅ Backface culling working correctly
- ✅ Lighting applied properly

### DirectX Backend  
- ✅ Cube renders with texturing
- ✅ Backface culling working
- ✅ Lighting applied
- 🔧 UV coordinates flipped (V axis) to match DirectX convention

## Known Differences

### Coordinate Systems
- **Vulkan**: Right-handed, Y-up, clip space Y: [-1, 1], Z: [0, 1]
- **DirectX**: Left-handed, Y-up, clip space Y: [-1, 1], Z: [0, 1]

### Texture Coordinates
- **Vulkan/OpenGL**: V=0 at bottom, V=1 at top
- **DirectX**: V=0 at top, V=1 at bottom
- **Fix**: Flip V coordinate in HLSL vertex shader: `output.uv = float2(input.uv.x, 1.0 - input.uv.y);`

### Camera Matrices
- **Vulkan**: `perspective_rh()` and `look_at_rh()`
- **DirectX**: `perspective_lh()` and `look_at_lh()`
- Implemented in `src/camera/mod.rs` with backend detection

## Testing
- Run Vulkan: `cargo run --release`
- Run DirectX: `./run_with_proton.sh`
- Both should show identical cube orientation and texturing

## Next Steps
- Remove hardcoded rendering paths
- Implement render pass templates from config
- Enable CI rendering tests
- Load all object data from GLTF (no hardcoded data)

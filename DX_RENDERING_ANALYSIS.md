# DirectX Rendering Analysis

## Date: 2025-11-16

## Current Status

### Working
- ✅ DX backend compiles and runs without crashes
- ✅ Cube scene renders correctly (simple inline mesh geometry)
- ✅ Rendering pipeline is functional
- ✅ Textures are loaded and bound correctly
- ✅ Command submission works without GPU faults

### Not Working
- ❌ glTF models (damaged helmet) render as black/clear color in DX
- ❌ Clear color mismatch (black in DX vs blue in VK)

## Test Results

### Cube Scene Test
- **DX**: Cube visible with correct maroon color at center
- **VK**: Cube visible with same maroon color
- **Difference**: Background color (black in DX, blue in VK)
- **RMSE**: 20072 (0.306) - mainly due to background difference

### Helmet Scene Test
- **DX**: Mostly black image, no helmet visible
- **VK**: Helmet renders correctly with textures
- **RMSE**: 18076.7 (0.276) - significant difference

## Technical Details

### Camera Matrix Difference
- **VK**: `Row 1: [0.0, -2.4142134, 0.0, 0.0]` (Y negated)
- **DX**: `Row 1: [0.0, 2.4142134, 0.0, 0.0]` (Y as-is)
- This is intentional per `src/camera/mod.rs` lines 52-66

### Logs Analysis
From DX log for helmet scene:
```
[INFO] Drawing 46356 indices
[INFO] Texture binding 2: 2048x2048, format: Rgba8Unorm, has SRV: true
[INFO] DrawIndexed completed: 46356 indices, 1 instances
```

Draw commands are issued correctly, but nothing appears on screen.

## Hypotheses

### Primary Hypothesis: Vertex Layout Issue
glTF models may have a different vertex layout than inline meshes, causing:
- Incorrect attribute interpretation
- Wrong stride/offset calculations
- Missing or incorrectly bound vertex attributes

### Secondary Hypotheses
1. **Depth Test Issue**: Helmet geometry might be failing depth test
2. **Winding Order**: Face culling might be backwards for glTF models in DX
3. **Transform/Matrix Issue**: Model-view-projection calculation might be wrong for glTF
4. **Shader Binding**: Root signature parameters might not match for glTF path

## Next Steps

1. Compare vertex layouts between cube and helmet in DX backend
2. Check if vertex attribute bindings match shader expectations
3. Verify model-view-projection matrix calculations for glTF models
4. Test with a simpler glTF model (textured cube)
5. Enable graphics debugging/validation to catch any state errors
6. Fix clear color to be consistent between backends

## Clear Color Issue

Current code in `src/backends/directx/dx12_impl.rs:2154`:
```rust
let clear_color = [0.0f32, 0.0f32, 0.0f32, 1.0f32]; // Hardcoded black
```

This should be pulled from render pass definition, not hardcoded. VK uses blue clear color.

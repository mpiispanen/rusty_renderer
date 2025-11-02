# Vulkan Rendering Debug Session - November 2, 2025

## Summary
Fixed critical shader/uniform buffer mismatches that were preventing Vulkan rendering from displaying geometry.

## Issues Fixed

### 1. Shader Material Buffer Mismatch
**Problem**: Shader referenced `MaterialUniforms` (b3) that wasn't being bound
**Solution**: Removed material buffer from shader, using only vertex colors

### 2. Push Constants Stage Flags
**Problem**: Pushing to VERTEX|FRAGMENT (0x11) when shader only uses VERTEX stage
**Solution**: Changed to VERTEX stage only (0x1)

### 3. Camera Uniforms Structure  
**Problem**: App sent 144 bytes (view + proj + view_pos), shader expected 64 bytes (viewProj)
**Solution**: Pre-multiply matrices and send single viewProj matrix

### 4. Lighting Uniforms Structure
**Problem**: App sent 48 bytes (simple structure), shader expected 400 bytes (ambient + lights array)
**Solution**: Restructured to match shader with full lights array support

## Files Modified

- `shaders/hlsl/forward_simple.hlsl` - Removed material buffer, simplified fragment shader
- `src/passes/forward_simple.rs` - Fixed stage flags, updated buffer sizes
- `src/app.rs` - Restructured uniform data, added matrix multiplication

## Next Steps

1. **Verify rendering** - Test that cube is now visible
2. **DirectX parity** - Apply same fixes to DirectX backend  
3. **Material support** - Add proper material buffer when needed
4. **Optimize** - Consider shader variants for different feature sets

## Technical Notes

### Uniform Buffer Layouts
All uniform buffers must match between HLSL and Rust:
- HLSL `cbuffer` size = Rust struct size
- Alignment and padding must match
- Array sizes must match exactly

### HLSL Register Mapping to Vulkan
- `register(b0)` → Binding 0 (uniform buffer)
- `register(b1)` → Binding 1 (uniform buffer)
- `register(b2)` → Push constants (special case)
- `register(t0)` → Binding N (texture/sampler)

This mapping is handled by DXC when compiling HLSL to SPIR-V for Vulkan.

# Vulkan Cube Rendering Debug Session - Part 5
## Date: 2025-11-02

## Summary

Investigated why cube rendering produces only clear color while triangle rendering works.

## What I Investigated

### 1. Buffer Management (Red Herring)
- Initially saw buffer "Dropping" messages during execution
- Traced these extensively and found they were:
  - Staging buffers being dropped after upload (correct behavior)
  - Final cleanup drops at end of program (correct behavior)
- **Conclusion**: Buffers are managed correctly, this was not the issue

### 2. Resource Lifecycle 
- Confirmed resources are allocated once
- Resources stay in HashMaps throughout rendering
- No premature cleanup happening

### 3. Rendering Execution
- Confirmed all rendering commands execute:
  - Vertex buffer binding
  - Uniform buffer binding (camera, lighting)  
  - Push constants
  - Draw call for 36 vertices
- No Vulkan validation errors (except unused UV attribute warning)

### 4. Shader Execution
- Modified pixel shader to return solid magenta
- Result: Still only clear color
- **Conclusion**: Pixel shader is NOT being executed at all

## Current Status

### What Works
- Triangle pass (uses hardcoded vertices in shader, no buffers)
- All Vulkan API calls complete without error
- Buffers are created, uploaded, and bound correctly

### What Doesn't Work
- Cube geometry doesn't render
- Pixel shader never executes (no fragments generated)
- Output is only clear color (RGB 51, 77, 128)

## Possible Causes

Since pixel shader doesn't run, fragments aren't being generated. Possible reasons:

1. **Vertices outside clip space**: Transform matrices pushing geometry off-screen
2. **Vertex shader issue**: Could be transforming vertices to invalid positions
3. **Viewport/Scissor mismatch**: Rendering area doesn't overlap framebuffer
4. **Depth buffer issue**: All fragments failing depth test
5. **Vertex buffer data corruption**: Despite successful upload, data might be wrong
6. **Descriptor set binding issue**: Uniforms not actually accessible to shader

Ruled out:
- ✅ Back-face culling (tested with CullMode::None)
- ✅ Buffer lifecycle issues
- ✅ Missing rendering commands

## Recommended Next Steps

1. **Check vertex shader output**:
   - Log/debug the transformed vertex positions
   - Verify they're in valid clip space (-1 to 1)
   
2. **Simplify the shader**:
   - Remove all uniform/push constant usage
   - Use hardcoded transform matrices
   - Verify basic vertex processing works

3. **Verify vertex buffer data**:
   - Dump the actual bytes uploaded to GPU
   - Confirm vertex layout matches shader expectations

4. **Test with simpler geometry**:
   - Try a single triangle using vertex buffer (not hardcoded)
   - Bridges the gap between working (triangle pass) and broken (cube pass)

5. **Add RenderDoc/validation**:
   - Capture a frame with RenderDoc
   - See actual vertex/uniform data GPU receives
   - Check what vertex shader outputs

## Files Modified

- `src/backends/vulkan/mod.rs`: Added extensive debug logging
- `src/backends/vulkan/resources.rs`: Added drop backtrace logging
- `src/passes/forward_simple.rs`: Changed CullMode::Back -> None for testing
- `src/app.rs`: Added debug logging

## Key Insights

The issue is NOT with resource management or API usage. The Vulkan commands are all correct. The problem is with the DATA or TRANSFORMS - something is causing zero fragments to be generated, which means the geometry is either:
- Not reaching the rasterizer
- Being transformed to invalid positions
- Being clipped entirely

The fact that the triangle pass works (with hardcoded vertices) but the cube pass doesn't (with vertex buffers and transforms) suggests the issue is in how we're using vertex buffers OR in the transformation matrices.

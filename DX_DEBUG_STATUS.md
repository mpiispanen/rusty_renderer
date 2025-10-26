# DirectX Rendering Debug Status
**Date:** 2025-10-26  
**Session:** DirectX Cube Rendering Investigation

## Summary

DirectX backend is executing correctly from an API perspective but not displaying the cube. All draw calls execute successfully, shaders compile, and no errors are reported.

## What's Working ✓

1. **Shader Compilation**: Forward rendering shader (forward.hlsl) compiles successfully
   - VSMain (vs_5_0): SUCCESS
   - PSMain (ps_5_0): SUCCESS
   
2. **API Execution**: All DirectX API calls execute without error
   - Device initialization: OK
   - Swapchain creation: OK (800x600, 3 images)
   - Pipeline creation: OK
   - Root signature: OK
   
3. **Draw Calls**: Drawing 36 vertices per frame
   - Vertex buffer created: 1728 bytes
   - Draw instances: 36 vertices, 1 instance
   - Ran 600 frames successfully
   
4. **Resource Binding**:
   - Push constants: 32 DWORDs (128 bytes) - model + normal matrices
   - Uniform buffers bound:
     - Camera (b0): 64 bytes at GPU addr 0xffff800102400000
     - Lighting (b1): 400 bytes at GPU addr 0xffff800102410000
     - Material (b3): 32 bytes at GPU addr 0xffff800102440000
   - Texture bound at set 0, binding 2
   
5. **Root Signature**: Matches shader expectations
   - Root param 0: Camera CBV (b0)
   - Root param 1: Lighting CBV (b1)
   - Root param 2: Push constants (b2) - 32 DWORDs
   - Root param 3: Material CBV (b3)
   - Root param 4: Texture descriptor table (t0)
   - Static sampler: s0

## What's Not Working ✗

1. **No Visual Output**: Cube is not visible, only clear color shown
   - Expected: Textured cube with lighting
   - Actual: Dark blue clear color only
   
2. **Vulkan Backface Rendering**: Vulkan shows backfaces instead of frontfaces
   - Suggests possible geometry winding order issue

## Configuration

### Pipeline State
- **Rasterizer**:
  - CullMode: BACK
  - FrontCounterClockwise: TRUE
  - FillMode: SOLID
  
- **Depth/Stencil**:
  - DepthEnable: TRUE
  - DepthFunc: LESS
  - Clear value: 1.0
  
- **Viewport**: 0,0 to 800x600
- **Scissor**: Full viewport
- **Topology**: TRIANGLELIST

### Vertex Format
Stride: 48 bytes
- Position: 3 floats (12 bytes) at offset 0
- Normal: 3 floats (12 bytes) at offset 12
- UV: 2 floats (8 bytes) at offset 24
- Color: 4 floats (16 bytes) at offset 32

### Input Layout
Matches shader VSInput:
```hlsl
struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};
```

## Potential Issues

### 1. Camera/Projection Matrix
- Cube may be outside view frustum
- Need to verify camera position and projection calculations
- DirectX uses different matrix conventions than Vulkan

### 2. Vertex Data Upload
- Buffer created with CpuToGpu memory
- Data should be uploaded via Map/Unmap
- Need to verify actual vertex data content

### 3. Coordinate System Differences
- DirectX uses Y-down screen space (Vulkan uses Y-up)
- May need to adjust projection matrix
- FrontCounterClockwise setting may need inversion

### 4. Texture Binding
- Texture is bound via descriptor table
- May need to verify descriptor heap setup
- Sampler is static in root signature

## Debug Logs

Latest execution (10 frames):
```
DirectX initialized successfully!
Loaded forward.hlsl shader (with textures)
Shader compilation SUCCESS for VSMain (vs_5_0)
Shader compilation SUCCESS for PSMain (ps_5_0)
...
DirectX Draw: 36 vertices, 1 instances
```

No errors, warnings, or validation layer messages.

## Next Steps

### Immediate
1. ✅ Embedded forward.hlsl shader to eliminate file loading issues
2. ⏭️ Add vertex data content logging to verify upload
3. ⏭️ Test with hardcoded triangle to verify pipeline works
4. ⏭️ Compare matrix calculations between Vulkan and DirectX

### Investigation Tools
1. **RenderDoc** or **PIX**: Capture frame to inspect:
   - Actual vertex buffer content
   - Matrix values in constant buffers
   - Pipeline state at draw time
   - Texture binding state
   
2. **Validation Layer**: Enable D3D12 debug layer for more details
   - Check for silent errors
   - Verify resource states
   
3. **Comparison Test**: Run same scene in Vulkan and DirectX
   - Compare vertex data
   - Compare uniform values
   - Compare projection matrices

### Vulkan Backface Fix
- Currently shows backfaces instead of frontfaces
- FrontFace set to COUNTER_CLOCKWISE (correct)
- May need to:
  - Reverse winding order in geometry
  - Or flip CullMode
  - Or adjust front face definition

## Files Modified

1. `src/backends/directx/dx12_impl.rs`:
   - Changed HLSL_SHADER_SOURCE to use `include_str!` for forward.hlsl
   - Ensured forward rendering shader is embedded
   
2. `run_with_proton.sh`:
   - Already configured for DirectX backend
   - Default scene: gltf_textured.toml
   - Default frames: 600

## Test Commands

```bash
# Vulkan (shows backfaces)
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --max-frames 3

# DirectX via Proton (no cube visible)
./run_with_proton.sh --max-frames 10

# Check DirectX debug log
tail -100 windows_test_directx/rusty_renderer_debug.log
```

## Conclusion

The DirectX backend is functionally correct at the API level but has a rendering issue preventing the cube from appearing. The most likely causes are camera/projection matrix issues or coordinate system mismatches. Further investigation with graphics debugging tools or detailed logging is needed to identify the exact problem.

Vulkan also needs a backface culling fix, suggesting the geometry data or winding order may need adjustment for both backends.

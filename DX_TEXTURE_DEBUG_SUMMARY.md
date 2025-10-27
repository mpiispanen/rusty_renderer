# DirectX Texture Issue - Debug Summary

## Problem
DirectX backend renders a black cube instead of showing the textured and lit cube that Vulkan renders correctly.

## Root Cause
Texture sampling in the pixel shader returns black (0,0,0,0) instead of the actual texture colors.

## Testing Results

### What Works ✓
1. **Geometry rendering** - Cube shape is correct
2. **Pixel shader execution** - PS runs and can output colors
3. **Uniform buffers** - baseColor uniform works (returns white 1,1,1)
4. **Vertex attributes** - Vertex colors work (white 1,1,1,1)
5. **Depth testing** - Cube faces render in correct order

### What Doesn't Work ✗
1. **Texture sampling** - `diffuseTexture.Sample(diffuseSampler, input.uv)` returns [0,0,0,0]

## Test Shader Results
```hlsl
// Test 1: return float4(1.0, 0.0, 1.0, 1.0);
Result: Magenta cube ✓ (PS works)

// Test 2: return float4(baseColor.rgb, 1.0);
Result: White cube ✓ (uniforms work)

// Test 3: return input.color;
Result: White cube ✓ (vertex data works)

// Test 4: return diffuseTexture.Sample(diffuseSampler, input.uv);
Result: BLACK cube ✗ (texture sampling fails)
```

## Comparison with Vulkan
- **Vulkan**: Center pixel [169, 154, 135] - correct checkerboard texture
- **DirectX**: Center pixel [0, 0, 0] - black from failed texture sample

## Likely Issues

###  1. Texture Not Uploaded to GPU
Check in `create_texture()` if texture data is actually uploaded.

### 2. SRV Not Created or Invalid
Check if `srv_gpu_handle` in DirectXTexture is valid.

### 3. Descriptor Heap Not Bound
Check if `SetDescriptorHeaps()` is called before `SetGraphicsRootDescriptorTable()`.

### 4. Wrong Root Parameter Index
Verify root parameter 4 matches the shader's `register(t0)`.

### 5. Texture State/Layout
Check if texture is in correct D3D12 resource state for shader reading.

## Next Steps

1. Add debug logging in `bind_texture()` to verify:
   - Texture pointer is valid
   - SRV GPU handle exists
   - Descriptor heap is set
   - Root descriptor table is bound

2. Check texture upload in `create_texture()`:
   - Verify `initial_data` is being uploaded
   - Check resource state transitions
   - Verify texture format matches

3. Compare with Vulkan's working texture binding to ensure parity

## Workaround
For now, can disable texture sampling in shader to at least see lit geometry:
```hlsl
// Temporarily skip texture sampling
float3 color = baseColor.rgb * input.color.rgb;
// Apply lighting...
```

## Files to Check
- `src/backends/directx/dx12_impl.rs` - `create_texture()`, `bind_texture()`
- `src/passes/forward.rs` - Texture binding in pass execution
- `shaders/hlsl/forward.hlsl` - Shader expects texture at t0


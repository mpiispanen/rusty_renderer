# DirectX Texture Debugging Session - 2025-10-27

## Current Status
DirectX backend renders a cube, but it appears black/gray instead of textured.

## Investigation Results

### ✅ Confirmed Working
1. **Material Buffer Creation**: Material buffer is created with correct data
   - `hasTexture = 1.0` (properties.z)
   - `baseColor = [1.0, 1.0, 1.0]`
   - Buffer uploaded before ForwardPass creation

2. **Material Buffer Binding**: Correct buffer is bound
   - Buffer pointer: `0x76ae00` (example from last run)
   - Bound to root parameter 3 (register b3)
   - GPU address: `0xffff800102440000`

3. **Texture Creation**: Texture is created successfully
   - Format: Rgba8Unorm
   - Size: 256x256
   - SRV created: YES

4. **Texture Binding**: Texture is bound correctly
   - Set 0, binding 2
   - Root parameter 4 (descriptor table for t0)
   - Descriptor heap is set before binding

5. **Root Signature**: Correctly defined
   - Root param 0: Camera (b0)
   - Root param 1: Lighting (b1)
   - Root param 2: Push constants (b2 - model/normal matrices)
   - Root param 3: Material (b3)
   - Root param 4: Texture descriptor table (t0)
   - Static sampler 0: s0

### ❓ Potential Issues

1. **Vertex Colors**: Might be black, which would darken the output when multiplied
   - Shader: `albedo *= input.color.rgb;`
   - Need to verify vertex color values

2. **Lighting**: Ambient might be too low
   - Current ambient: `[0.2, 0.2, 0.2]`
   - After multiplication, could make texture very dark

3. **Descriptor Heap Persistence**: Descriptor heap is set in `bind_texture()` but might need to be set again before draw

4. **Backface Culling**: User mentioned Vulkan was showing backfaces - need to verify winding order matches

## Debug Logs Show
```
GpuMaterial created: base_color=[1.00, 1.00, 1.00], has_texture=1, texture_path=Some(...)
Creating ForwardPass for 'TexturedCube' with material buffer at 0x76ae00
bind_uniform_buffer called: set=0, binding=3, buffer_ptr=0x76ae00
Binding uniform: set=0, binding=3, root_param=3, gpu_addr=0xffff800102440000, size=32
bind_texture called: set=0, binding=2
Texture: 256x256, format: Rgba8Unorm, has SRV: true
Binding texture with GPU handle ptr: 12884901952
Set descriptor heap
SetGraphicsRootDescriptorTable(4, gpu_handle) called
DirectX Draw: 36 vertices, 1 instances
```

## Next Steps
1. Check vertex color values in GLTF data
2. Simplify shader to test each component independently:
   - Just texture color
   - Just vertex color
   - Just base color
3. Verify descriptor heap persists through draw call
4. Check if there's a synchronization issue with buffer uploads

## Files Modified
- `/var/home/matpii01/rusty_renderer/src/pipelines/forward.rs` - Added debug logging
- `/var/home/matpii01/rusty_renderer/src/backends/directx/dx12_impl.rs` - Added debug logging
- `/var/home/matpii01/rusty_renderer/run_with_proton.sh` - Fixed default scene path


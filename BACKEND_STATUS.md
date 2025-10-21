# Backend Status - October 21, 2025

## Summary

**Vulkan:** ✅ Fully working with lighting
**wgpu:** ❌ Broken (push constants not implemented)
**DirectX:** ❓ Untested

---

## Vulkan Backend - ✅ COMPLETE

### Working Features
- Forward rendering with lighting
- Per-frame descriptor sets
- Push constants for transforms
- Camera uniforms (view-projection)
- Lighting uniforms (up to 8 lights)
- Vertex buffers
- Index buffers
- Headless and windowed modes
- Screenshot capture
- Zero validation errors

### Performance
- 50+ FPS easily achieved
- Proper synchronization
- Efficient per-frame resources

---

## wgpu Backend - ❌ BROKEN

### Status
Push constants are stubbed out - not implemented.

### The Issue
```rust
fn push_constants(&mut self, ...) -> Result<()> {
    log::debug!("WgpuPassContext: Push constants not yet implemented (stub)");
    // TODO: Implement push constants for wgpu backend
    Ok(())
}
```

### Why It's Broken
- Forward rendering requires push constants for model/normal matrices
- Without them, transforms don't work
- Likely renders garbage or crashes

### How to Fix (~1-2 hours)

**Option 1: Dynamic Uniforms** (Recommended)
wgpu doesn't have push constants like Vulkan. Use dynamic uniform buffers instead:

1. Create a uniform buffer for transforms
2. Use dynamic offsets when binding
3. Update buffer with transform data before each draw

```rust
// Pseudo-code
impl WgpuPassContext {
    fn push_constants(&mut self, stage_flags: u32, offset: u32, data: &[u8]) -> Result<()> {
        // Write data to a dynamic uniform buffer
        let transform_buffer = self.get_or_create_transform_buffer();
        self.queue.write_buffer(&transform_buffer, offset as u64, data);
        
        // Bind with dynamic offset in next draw call
        self.pending_transform_offset = Some(offset);
        Ok(())
    }
}
```

**Option 2: Per-Object Uniform Buffers**
Create a separate uniform buffer for each object's transforms. Less efficient but simpler.

### Testing wgpu
Once implemented:
```bash
cargo run --features wgpu -- --scene scenes/cube.toml --pipeline forward
```

---

## DirectX Backend - ❓ UNTESTED

### Status
Push constants stubbed, but DirectX implementation exists.

### The Issue
```rust
fn push_constants(&mut self, ...) -> Result<()> {
    log::debug!("DirectXPassContext: Push constants not yet implemented (stub)");
    // TODO: Implement push constants for DirectX backend
    // DirectX uses root constants instead of push constants
    Ok(())
}
```

### DirectX Equivalent
DirectX 12 uses **Root Constants** instead of push constants:

1. Define root constants in root signature
2. Use `SetGraphicsRoot32BitConstants()` to upload data
3. Very similar to Vulkan push constants

### How to Fix (~1-2 hours)

1. **Update Root Signature**
```rust
// Add root constants to root signature
// 32 x 32-bit values = 128 bytes (our transform data)
D3D12_ROOT_PARAMETER rootParams[] = {
    // Existing descriptor tables...
    
    // Add root constants
    {
        .ParameterType = D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
        .Constants = {
            .ShaderRegister = 0,      // register(b0) in shader
            .RegisterSpace = 1,        // space1
            .Num32BitValues = 32,      // 128 bytes / 4
        },
        .ShaderVisibility = D3D12_SHADER_VISIBILITY_VERTEX,
    },
};
```

2. **Implement push_constants**
```rust
fn push_constants(&mut self, stage_flags: u32, offset: u32, data: &[u8]) -> Result<()> {
    let command_list = self.command_list();
    
    // Convert bytes to u32 array
    let num_values = data.len() / 4;
    let values = unsafe {
        std::slice::from_raw_parts(data.as_ptr() as *const u32, num_values)
    };
    
    unsafe {
        command_list.SetGraphicsRoot32BitConstants(
            ROOT_PARAMETER_INDEX,  // Index in root signature
            num_values as u32,
            values.as_ptr() as *const _,
            offset / 4,  // Offset in 32-bit units
        );
    }
    
    Ok(())
}
```

3. **Update Shader**
DirectX shaders use different syntax:
```hlsl
// Instead of push constants:
cbuffer PushConstants : register(b0, space1) {
    float4x4 model;
    float4x4 normalMatrix;
};
```

### Testing DirectX
Requires Windows:
```bash
cargo run --features directx -- --scene scenes/cube.toml --pipeline forward
```

---

## Priority

### High Priority
**wgpu** - More important for cross-platform support
- Works on web (WebGPU)
- Works on all desktop platforms
- Good fallback when Vulkan unavailable

### Medium Priority  
**DirectX** - Important for Windows
- Native on Windows
- Good performance
- Already partially implemented

---

## Estimated Time to Fix

- **wgpu:** 1-2 hours (dynamic uniforms approach)
- **DirectX:** 1-2 hours (root constants)
- **Total:** ~3-4 hours for both backends

---

## Current Recommendation

**For now:** Use Vulkan backend - it's complete and working perfectly!

**When needed:** Implement wgpu first (broader platform support), then DirectX.

---

## Related Files

### wgpu
- `src/backends/wgpu_backend/mod.rs` - Main backend implementation
- Need to add dynamic uniform buffer support

### DirectX
- `src/backends/directx/dx12_impl.rs` - DX12 implementation
- `src/backends/directx/mod.rs` - Wrapper
- Need to update root signature and implement root constants

### Shaders
- `shaders/forward.vert` - Currently GLSL (Vulkan)
- Would need HLSL version for DirectX
- wgpu can use WGSL or SPIR-V

---

**Updated:** 2025-10-21 21:37 UTC

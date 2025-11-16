# DirectX Rendering Debug Action Plan
**Date:** November 15, 2025  
**Status:** GPU fault at address 0x00000000 - NULL pointer access from shader

## Current Situation

### What Works
- ✅ Logging infrastructure cleaned up (using `fern` for dual stdout+file output)
- ✅ Vulkan backend renders correctly with validation layers
- ✅ Build system working
- ✅ Scene loading and default texture creation in place

### What's Broken
- ❌ DirectX backend causes GPU VM fault when running under Proton
- ❌ GPU tries to read from address 0x00000000 (NULL pointer)
- ❌ Device lost after GPU fault
- ❌ Application may hang/timeout

## Root Cause Analysis

The GPU fault signature is clear:
```
radv: GPUVM fault detected at address 0x00000000.
CLIENT_ID: (SQC (data)) 0xa    <- Shader Quad Cache trying to read data
PERMISSION_FAULTS: 3            <- 3 failed access attempts
```

**This means:** The shader is trying to sample a texture that doesn't have a valid GPU descriptor bound.

## Key Questions

1. **Are textures being uploaded to GPU memory?**
   - Check if `upload_texture_data()` completes successfully
   - Verify GPU virtual address is non-zero
   
2. **Are SRVs (Shader Resource Views) created correctly?**
   - Verify descriptor heap has space
   - Check GPU handle offsets are correct
   
3. **Are descriptor heaps bound before draw calls?**
   - Ensure `SetDescriptorHeaps()` called before `SetGraphicsRootDescriptorTable()`
   - Verify heap binding happens every frame
   
4. **Is texture upload synchronized properly?**
   - Upload uses separate command allocator
   - Must wait for upload fence before rendering samples texture
   - Check if `upload_allocator_fence_value` is properly tracked

5. **Are resource states correct?**
   - Texture transitions: COMMON → COPY_DEST → PIXEL_SHADER_RESOURCE
   - Barriers must execute before texture is sampled

## Debugging Steps

### Step 1: Add Comprehensive Logging

Add debug logging at critical points (already using proper `log` crate):

**In `create_texture()`:**
```rust
log::info!("Creating texture: {}x{}, format: {:?}", width, height, format);
log::info!("  SRV heap offset: {}", descriptor_heap_offset);
log::info!("  GPU handle: {:?}", gpu_handle);
```

**In `upload_texture_data()`:**
```rust
log::info!("Uploading texture data: {} bytes", data.len());
log::info!("  Upload fence value before: {}", self.upload_allocator_fence_value);
log::info!("  Signaling fence: {}", self.fence_value + 1);
log::info!("  Upload fence value after: {}", self.upload_allocator_fence_value);
```

**In `execute_graph()` before drawing:**
```rust
log::info!("Setting descriptor heaps before draw");
log::info!("  Heap count: 1");
log::info!("Binding root descriptor table");
log::info!("  Root parameter index: 1");
log::info!("  Base GPU descriptor: <value>");
```

### Step 2: Validate Synchronization

**Add explicit fence wait after texture upload:**
```rust
// In upload_texture_data(), after ExecuteCommandLists:
let fence_to_wait = self.fence_value;
self.wait_for_gpu(fence_to_wait)?;  // Block until upload completes
log::info!("Texture upload completed and GPU synchronized");
```

### Step 3: Check Descriptor Heap Binding

Verify descriptor heaps are bound every frame:
```rust
// In execute_graph(), before any draws:
let heaps = [Some(self.descriptor_heap.clone())];
command_list.SetDescriptorHeaps(&heaps);
log::debug!("Descriptor heaps bound for this command list");
```

### Step 4: Validate Draw Parameters

Log everything about the draw call:
```rust
log::info!("Draw call parameters:");
log::info!("  Vertex buffer: bound={}, size={}", vb_bound, vb_size);
log::info!("  Index buffer: bound={}, count={}", ib_bound, index_count);
log::info!("  Texture: bound={}, srv_offset={}", tex_bound, srv_offset);
log::info!("  Pipeline: {:?}", pipeline);
```

### Step 5: Test with Minimal Scene

Create a test that bypasses texture loading:
- Single triangle
- Solid color (no texture sampling)
- Verify geometry renders at all

### Step 6: Check Resource Creation Order

Ensure this sequence:
1. Create texture resource
2. Create SRV in descriptor heap
3. Upload data to texture
4. **WAIT FOR UPLOAD FENCE**
5. Transition to PIXEL_SHADER_RESOURCE
6. **WAIT FOR BARRIER**
7. Bind descriptor heap
8. Set root descriptor table
9. Draw

## Suspected Issues

### Most Likely: Missing Synchronization
The upload command allocator uses a separate fence value (`upload_allocator_fence_value`), but the main render loop may not wait for it. This means:
- Texture upload submitted to GPU
- Render starts immediately
- Shader samples texture before upload completes
- GPU accesses uninitialized memory (appears as NULL)

**Fix:** Add explicit fence wait after texture upload before returning from `upload_texture_data()`.

### Second Most Likely: Descriptor Heap Not Bound
The descriptor heap containing SRVs must be bound to the command list before setting root descriptor tables. If forgotten:
- Root descriptor table points to unbound heap
- GPU can't resolve texture descriptors
- Shader samples from invalid location (NULL)

**Fix:** Ensure `SetDescriptorHeaps()` is called in `execute_graph()` before any draw that uses textures.

### Third: Wrong Descriptor Offset
The `descriptor_heap_offset` is incremented each time a texture is created, but:
- May not account for reserved descriptors
- Root descriptor table may use wrong offset
- GPU reads wrong descriptor (possibly empty/NULL)

**Fix:** Add validation that offset < heap size, log actual GPU handle values.

## Validation Layers

DirectX debug layer is enabled when `--debug` flag is passed, but under Proton it may not work. Native Windows testing would help.

**Alternative:** Add manual validation:
- Check all pointers are non-NULL before use
- Verify GPU handles are within heap range
- Validate fence values are increasing monotonically
- Check resource states match expected states

## Files to Modify

### Primary
- `src/backends/directx/dx12_impl.rs`
  - Line ~2911: `create_texture()` - Add logging
  - Line ~3160: `upload_texture_data()` - Add fence wait
  - Line ~2198: `execute_graph()` - Verify heap binding
  
### Testing
- Run with: `./run_with_proton.sh --scene damaged_helmet --headless --max-frames 3 --debug`
- Check: `windows_test_directx/rusty_renderer.log`
- Look for: GPU faults, fence values, descriptor heap binding

## Success Criteria

- No GPU VM faults
- No "address 0x00000000" errors
- Rendered output shows textured model
- Log shows proper synchronization:
  - Texture upload completes before render
  - Descriptor heaps bound before draw
  - Fence values increasing monotonically
- Performance acceptable (no excessive waits)

## Next Immediate Action

1. Add logging to `upload_texture_data()` to track fence synchronization
2. Add explicit wait after texture upload
3. Add logging to `execute_graph()` to verify heap binding
4. Test and check logs for patterns
5. Fix identified issues iteratively

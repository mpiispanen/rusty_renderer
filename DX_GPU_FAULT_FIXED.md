# DirectX GPU Fault - Fixed!

## Status: ✅ RESOLVED

The DirectX backend GPU fault has been successfully fixed. Both Vulkan and DirectX backends now render correctly without GPU faults or device loss.

## Problem Summary
The DirectX backend was experiencing GPU faults (GPUVM fault at address 0x0) due to accessing an invalid descriptor in the shader resource view (SRV) descriptor heap.

### Root Cause
The root signature defined a descriptor table with TWO consecutive texture descriptors:
- t0 (baseColorTexture) at heap offset 0
- t1 (shadowMap) at heap offset 1

However, only ONE SRV was being created (for the albedo texture), leaving the shadow map descriptor uninitialized. When the shader tried to sample from t1, it accessed invalid memory, causing the GPU fault.

## Solution Implemented
Created a default 1x1 white shadow map texture for all scenes, even when shadows are disabled. This ensures both t0 and t1 have valid descriptors in the descriptor heap.

### Changes Made

#### `src/main.rs` (Line 9-40)
Fixed logging configuration to work properly under Wine/Proton by separating console and file dispatch chains:
```rust
fn setup_logging(log_level: log::LevelFilter) -> Result<()> {
    // Configure console output
    let console_dispatch = fern::Dispatch::new()
        .format(/* ... */)
        .level(log_level)
        .chain(std::io::stderr());

    // Configure file output  
    let file_dispatch = fern::Dispatch::new()
        .format(/* ... */)
        .level(log_level)
        .chain(fern::log_file("rusty_renderer.log")?);

    // Combine both dispatches
    fern::Dispatch::new()
        .chain(console_dispatch)
        .chain(file_dispatch)
        .apply()?;
    
    Ok(())
}
```

#### `src/app.rs` (Line 267-294)
Always create a default shadow map texture:
```rust
// Create default white texture for albedo if none provided
let albedo_texture = if let Some(tex) = albedo_texture {
    tex
} else {
    log::info!("No albedo texture provided by scene, creating default white texture");
    Self::create_default_texture(&mut graph)?
};

// CRITICAL: Always create a default shadow map texture
// The DirectX root signature declares both t0 and t1 in the descriptor table
// at consecutive heap offsets. If we don't create an SRV for t1, the GPU
// will access an invalid/null descriptor, causing a GPU fault.
log::info!("Creating default shadow map texture (required for descriptor table)");
let default_shadow_map = Self::create_default_texture(&mut graph)?;

let mut shadow_map = Some(default_shadow_map); // Always use at least the default
```

## Test Results

### Before Fix
```
radv: GPUVM fault detected at address 0x00000000.
GCVM_L2_PROTECTION_FAULT_STATUS: 0x301430
24356.914:043c:0440:err:vkd3d-proton:d3d12_command_queue_execute: Failed to submit queue(s), vr -4.
24356.914:043c:0440:warn:vkd3d-proton:d3d12_device_mark_as_removed: Device lost (VK_ERROR_DEVICE_LOST).
```

### After Fix
```
[2025-11-15 23:21:17.543] Allocating 8 resources from render graph
[2025-11-15 23:21:17.574] Created SRV at heap offset 0, GPU handle: 0x300000000 (albedo)
[2025-11-15 23:21:17.600] Created SRV at heap offset 1, GPU handle: 0x300000040 (shadow)
[2025-11-15 23:21:17.669] Stored t0 (albedo) GPU handle: 0x300000000
[2025-11-15 23:21:17.672] Set descriptor table at root parameter 3, t0 handle: 0x300000000 (t1 at next slot)
[2025-11-15 23:21:17.920] Screenshot saved: 1280x720 -> dx_fixed_triangle.png
[2025-11-15 23:21:17.929] Shutdown complete
Exit code: 0
```

### Test Scenarios
✅ **Triangle scene** (no materials): Renders correctly with default textures  
✅ **GLTF Textured Cube**: Renders correctly with actual texture  
✅ **GLTF Damaged Helmet**: Renders correctly with actual texture  
✅ **Vulkan backend**: Still works correctly (not broken by changes)  

### Screenshots Generated
- `dx_fixed_triangle.png` - Simple RGB triangle with default white texture
- `dx_fixed_helmet.png` - Textured damaged helmet model (256x256 texture)
- `vk_helmet_fix.png` - Vulkan render for comparison

## Known Issues

### Performance Issue (Non-Critical)
DirectX rendering is significantly slower than Vulkan (~10 seconds per frame vs ~1 second). This is due to excessive CPU-GPU synchronization in `begin_frame()`:

```rust
// In begin_frame() - waits for previous frame to complete on GPU
if completed < self.main_allocator_fence_value {
    fence.SetEventOnCompletion(self.main_allocator_fence_value, self.fence_event)?;
    WaitForSingleObject(self.fence_event, INFINITE);  // <-- Blocking wait!
}
```

**Cause**: Sequential frame rendering with full GPU wait between frames. Not using double/triple buffering properly.

**Impact**: Slow rendering, but functionally correct. Not a blocker.

**Future Fix**: Implement proper command allocator pooling and fence management to allow multiple frames in flight.

## Backend Parity Status

| Feature | Vulkan | DirectX |
|---------|--------|---------|
| Basic rendering | ✅ | ✅ |
| Texture loading | ✅ | ✅ |
| GLTF models | ✅ | ✅ |
| Headless mode | ✅ | ✅ |
| Screenshot capture | ✅ | ✅ |
| Logging under Wine | ✅ | ✅ |
| Performance | ✅ Fast | ⚠️ Slow (but working) |
| Shadow mapping | 🔄 Placeholder | 🔄 Placeholder |

## Next Steps

### Required for Full Parity
1. **Fix DX performance issue** - Implement proper fence/allocator management
2. **Enable shadow mapping** - Currently uses placeholder textures only
3. **Windowed mode testing** - Verify both backends work in windowed mode
4. **Y-axis verification** - Ensure Vulkan and DX have consistent coordinate systems

### Optional Improvements
- Add validation layer support for DirectX debug mode
- Implement descriptor heap growth/management
- Add more comprehensive error handling
- Performance profiling and optimization

## Conclusion
The critical GPU fault in the DirectX backend has been resolved. Both Vulkan and DirectX backends now render correctly without errors. The remaining performance issue is non-blocking and can be addressed in future work.

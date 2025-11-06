# Render Pass Architecture Investigation - Summary

## Date: 2025-11-06

## Problem Identified

While working on implementing shadow mapping (Issue #90), we discovered a critical architectural issue with the render graph backend implementation.

## Root Cause

Both Vulkan and DirectX backends violate the render graph modularity principle by sharing framebuffers and depth buffers across all passes:

### Vulkan (`src/backends/vulkan/mod.rs:3186-3220`)
```rust
// BEGIN ONE render pass for entire frame
let framebuffer = self.framebuffers[image_index];
let render_pass_info = vk::RenderPassBeginInfo::builder()
    .render_pass(self.render_pass)
    .framebuffer(framebuffer)  // Same framebuffer for ALL passes
    ...
device.cmd_begin_render_pass(...)

// Execute ALL passes within this single render pass
for pass_id in &compiled.execution_order {
    // All passes share the same color + depth attachments
    ...
}

// END render pass
device.cmd_end_render_pass(command_buffer);
```

### DirectX (`src/backends/directx/dx12_impl.rs:2327`)
```rust
// Set ONE render target for ALL passes
command_list.OMSetRenderTargets(1, Some(&rtv_handle), FALSE, Some(&dsv_handle));

// Execute ALL passes with the same render targets
for pass_id in &compiled.execution_order {
    // All passes share the same RTV and DSV
    ...
}
```

## Consequences

1. **Shadow mapping artifacts**: Shadow pass and forward pass both write to the same depth buffer, causing ghost geometry
2. **No resource transitions**: Resources can't transition between different layouts/states between passes
3. **Can't compose techniques**: Deferred rendering, post-processing chains, etc. won't work properly
4. **Violates render graph design**: The whole point of a render graph is modular, composable passes

## Correct Architecture

Each pass should:
1. Have its own framebuffer based on its declared outputs
2. Begin and end its own render pass
3. Transition resources between passes via barriers
4. Be completely independent and reusable

### Example: Shadow Map Pass
```
Outputs: shadow_map (1024x1024 depth texture)

Execution:
1. Create framebuffer with shadow_map as depth attachment (no color)
2. Begin render pass
3. Render from light's perspective
4. End render pass  
5. Transition shadow_map: DepthStencilAttachment → ShaderReadOnly
```

### Example: Forward Pass
```
Inputs:  shadow_map (texture)
Outputs: swapchain_image (color), forward_pass_depth (depth)

Execution:
1. Create framebuffer with swapchain_image + forward_pass_depth
2. Begin render pass
3. Bind shadow_map as texture sampler
4. Render with shadows
5. End render pass
```

## Solution

See `docs/render_pass_modularity.md` for detailed implementation plan.

### Phase 1: Vulkan
- Dynamic framebuffer creation from pass outputs
- Per-pass render pass begin/end
- Proper barrier insertion between passes

### Phase 2: DirectX  
- Per-pass RTV/DSV binding
- Resource state tracking
- Barrier insertion for state transitions

### Phase 3: Resource Management
- Lifetime tracking
- Automatic layout/state inference

## Current Status

- ✅ Problem identified and documented
- ✅ Shadow mapping temporarily disabled (commit daaf538)
- ✅ Detailed analysis in `docs/render_pass_modularity.md`
- ✅ Issue #94 created for tracking the fix
- ⏸️  Shadow mapping blocked on #94
- ⏸️  Other multi-pass techniques also blocked

## Testing

Once fixed, test with:
1. Shadow mapping (shadow + forward passes)
2. Post-processing chains (multiple color passes)
3. Deferred rendering (geometry + lighting passes)

## Impact

**High Priority** - Blocks:
- Shadow mapping (#90)
- Deferred rendering  
- Post-processing effects
- Any multi-pass rendering technique

## Files Modified

- `src/app.rs`: Disabled shadow mapping (lines 243-257)
- `docs/render_pass_modularity.md`: Detailed architectural analysis
- Issue #94: Tracking issue for the fix
- Issue #90: Updated with current status

## Next Steps

1. Implement per-pass framebuffer support in Vulkan (#94 Phase 1)
2. Implement per-pass render targets in DirectX (#94 Phase 2)
3. Re-enable shadow mapping
4. Test multi-pass rendering

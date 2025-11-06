# Render Pass Architecture Fix

## Problem

The current render graph implementation has a fundamental flaw: **passes share render targets incorrectly**, causing artifacts and breaking the modularity principle.

### Observed Issue
When both shadow map pass and forward pass are enabled:
- An artifact shaped like the model appears at a fixed screen position
- The artifact has the clear screen color
- Moving the camera doesn't affect the artifact
- Disabling the depth pass removes the artifact

### Root Cause
The shadow map pass writes to a depth texture, but this depth texture is also being used as the forward pass's depth attachment. This violates the principle that:

> **Each render pass must have completely independent render targets.**

When a pass reads from a previous pass's output (e.g., forward pass reading shadow map), it should:
1. **Read** from the previous pass's output as a texture/sampler
2. **Write** to its OWN separate render targets

## Current (Broken) Architecture

```
Shadow Map Pass:
  Output: shadow_depth_map (as depth attachment)

Forward Pass:
  Input:  shadow_depth_map (as sampled texture) 
  Output: color_buffer (as color attachment)
  Output: shadow_depth_map (as depth attachment)  ← WRONG! Reusing input as output
```

The forward pass is trying to use `shadow_depth_map` both as:
- An input (sampled depth texture for shadow calculations)
- An output (depth attachment for depth testing)

This creates a read-write hazard and violates resource barriers.

## Correct Architecture

```
Shadow Map Pass:
  Output: shadow_depth_map (as depth attachment)

Forward Pass:
  Input:  shadow_depth_map (as sampled texture) 
  Output: forward_color_buffer (as color attachment)
  Output: forward_depth_buffer (as depth attachment)  ← Separate depth buffer!
```

Key principles:
1. **Each pass has its own render targets** (color/depth attachments)
2. **Pass outputs become inputs to subsequent passes** (as textures/buffers)
3. **No render target is both read and written in the same pass**
4. **Resource transitions are handled by barriers between passes**

## Current Status

### What's Fixed (Code-Level)
✅ Resource declarations are now correct:
- Shadow pass has its own depth buffer (`shadow_map`)
- Forward pass has its own depth buffer (`forward_pass_depth`) 
- Forward pass correctly reads shadow_map as input texture
- Render graph correctly identifies dependencies

### What's Still Broken (Backend-Level)
❌ Vulkan backend uses ONE render pass for entire frame
❌ Vulkan backend uses ONE framebuffer for all passes
❌ DX12 backend has the same issue
❌ Barriers are calculated but not properly applied between passes

This means:
- Both passes render to the SAME swapchain framebuffer
- Shadow map writes are overwritten by forward pass
- Depth buffers are shared incorrectly
- Resource transitions don't happen properly

## Implementation Changes Required

## Implementation Changes Required

### 1. Backend: Per-Pass Framebuffers

**Current (Broken):**
```rust
// In execute_graph:
let framebuffer = self.framebuffers[image_index];  // Single framebuffer
device.cmd_begin_render_pass(...);                  // Single render pass
for pass_id in &compiled.execution_order {
    // All passes draw to same framebuffer
    callback.execute(&mut context);
}
device.cmd_end_render_pass();                       // End single render pass
```

**Required:**
```rust
// In execute_graph:
for pass_id in &compiled.execution_order {
    // Get pass's output resources
    let pass_outputs = graph.get_pass(*pass_id).outputs;
    
    // Create/get framebuffer for this pass's outputs
    let framebuffer = self.get_or_create_pass_framebuffer(pass_outputs);
    
    // Begin render pass for THIS pass only
    device.cmd_begin_render_pass(framebuffer, ...);
    
    // Execute THIS pass
    callback.execute(&mut context);
    
    // End render pass for THIS pass
    device.cmd_end_render_pass();
    
    // Insert barriers before next pass
    self.insert_barriers_after_pass(*pass_id, next_pass_id);
}
```

### 2. Backend: Dynamic Framebuffer Creation

Add new method to backend:
```rust
impl VulkanBackend {
    fn get_or_create_pass_framebuffer(
        &mut self,
        outputs: &[ResourceAccess]
    ) -> vk::Framebuffer {
        // 1. Collect image views from output resources
        let mut attachments = Vec::new();
        for output in outputs {
            match output.layout {
                Some(ImageLayout::ColorAttachment) => {
                    let image_view = self.resource_textures.get(&output.resource)
                        .and_then(|tex| tex.image_view());
                    attachments.push(image_view);
                }
                Some(ImageLayout::DepthStencilAttachment) => {
                    let depth_view = self.resource_textures.get(&output.resource)
                        .and_then(|tex| tex.image_view());
                    attachments.push(depth_view);
                }
                _ => {}
            }
        }
        
        // 2. Create or retrieve cached framebuffer
        // Cache key: hash of (attachment image views)
        let cache_key = calculate_framebuffer_hash(&attachments);
        if let Some(&fb) = self.framebuffer_cache.get(&cache_key) {
            return fb;
        }
        
        // 3. Create new framebuffer
        let framebuffer = create_vulkan_framebuffer(&attachments, ...);
        self.framebuffer_cache.insert(cache_key, framebuffer);
        framebuffer
    }
}
```

### 3. Backend: Per-Pass Render Pass Creation

Each pass needs its own Vulkan `RenderPass` object:

```rust
fn get_or_create_render_pass(
    &mut self,
    outputs: &[ResourceAccess]
) -> vk::RenderPass {
    // Create render pass description from outputs
    let mut color_attachments = Vec::new();
    let mut depth_attachment = None;
    
    for output in outputs {
        match output.layout {
            Some(ImageLayout::ColorAttachment) => {
                let resource = self.graph.get_resource(output.resource);
                color_attachments.push(ColorAttachmentDesc {
                    format: resource.format,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                });
            }
            Some(ImageLayout::DepthStencilAttachment) => {
                let resource = self.graph.get_resource(output.resource);
                depth_attachment = Some(DepthAttachmentDesc {
                    format: resource.format,
                    load_op: LoadOp::Clear,
                    store_op: StoreOp::Store,
                });
            }
            _ => {}
        }
    }
    
    // Cache and return render pass
    create_or_get_cached_render_pass(color_attachments, depth_attachment)
}
```

### 4. Backend: Proper Barrier Insertion

```rust
fn insert_barriers_between_passes(
    &mut self,
    cmd_buffer: vk::CommandBuffer,
    src_pass: PassId,
    dst_pass: PassId,
    barrier: &Barrier,
) {
    for image_barrier in &barrier.image_barriers {
        // Transition image from src layout to dst layout
        let image = self.resource_textures.get(&image_barrier.resource)
            .expect("Image not found").vk_image();
        
        vk::ImageMemoryBarrier::builder()
            .src_access_mask(to_vk_access(image_barrier.src_access))
            .dst_access_mask(to_vk_access(image_barrier.dst_access))
            .old_layout(to_vk_layout(image_barrier.old_layout))
            .new_layout(to_vk_layout(image_barrier.new_layout))
            .image(image)
            .subresource_range(...);
        
        device.cmd_pipeline_barrier(
            cmd_buffer,
            to_vk_stage(barrier.src_stage),
            to_vk_stage(barrier.dst_stage),
            0,
            &[],
            &[],
            &[image_barrier],
        );
    }
}
```

### 5. Resource Naming Convention
Adopt clear naming to show ownership:
- `shadow_pass_depth` - owned by shadow pass
- `forward_pass_color` - owned by forward pass  
- `forward_pass_depth` - owned by forward pass
- `final_output` - final swapchain image

### 2. Pass Declaration Changes

**Shadow Map Pass:**
```rust
// Outputs
- Write: shadow_pass_depth (ImageLayout::DepthStencilAttachment)

// No color output (depth-only pass)
```

**Forward Pass:**
```rust
// Inputs
- Read: shadow_pass_depth (ImageLayout::ShaderReadOnly)
  → Sampled in fragment shader for shadow calculations

// Outputs  
- Write: forward_pass_color (ImageLayout::ColorAttachment)
- Write: forward_pass_depth (ImageLayout::DepthStencilAttachment)
```

### 3. Barrier Insertion
The render graph must insert a barrier between shadow and forward passes:

```
After Shadow Pass:
  shadow_pass_depth: DepthStencilAttachment → ShaderReadOnly
  
Before Forward Pass:
  shadow_pass_depth: ShaderReadOnly (for sampling)
```

### 4. Backend Execution Changes

Each backend must:
1. **Start separate render passes** for each graph pass
2. **Only bind outputs as attachments** (never inputs)
3. **Bind inputs as descriptors** (uniform buffers, textures, etc.)
4. **Execute resource transitions** between passes

## Benefits of Correct Architecture

1. **True Modularity**: Passes can be added/removed without affecting others
2. **Resource Safety**: No read-write hazards
3. **Barrier Optimization**: Clear dependency chains
4. **Multi-Backend**: Works correctly on Vulkan, DX12, Metal, etc.
5. **Future-Proof**: Enables advanced features like:
   - Multiple shadow maps
   - Deferred rendering
   - Post-processing effects
   - Compute shader integration

## Migration Path

### Phase 1: Fix Shadow + Forward (Current Priority)
- Create separate depth buffers for each pass
- Update resource declarations
- Fix backend render pass begin/end

### Phase 2: Generalize Pattern
- Create helper functions for common pass patterns
- Document best practices
- Add validation warnings for incorrect usage

### Phase 3: Advanced Features
- Multi-pass effects (bloom, SSAO, etc.)
- Render pass graph visualization
- Auto-optimization of resource lifetimes

## Testing Strategy

1. **Visual Verification**:
   - No artifacts with shadow mapping enabled
   - Shadows render correctly on geometry
   - Camera movement works smoothly

2. **Resource Validation**:
   - Check barrier insertion in logs
   - Verify each pass has independent attachments
   - Confirm no read-write hazards

3. **Multi-Backend**:
   - Test on Vulkan and DX12
   - Verify identical output
   - Check performance impact

## Related Issues

- Issue #XX: Shadow mapping artifacts
- Milestone: Render Graph Architecture
- Related: Barrier insertion, resource lifetimes


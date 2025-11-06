# Render Pass Modularity Fix

## Problem

The current render graph implementation violates modularity principles because all passes share the same framebuffer and depth buffer. This causes artifacts when multiple passes try to write to the same depth attachment.

### Current Implementation Issues

1. **Vulkan Backend** (`src/backends/vulkan/mod.rs:3186-3220`):
   - Begins ONE render pass for the entire frame
   - All passes execute within this single render pass
   - All passes share the same framebuffer (swapchain color + main depth buffer)
   - Shadow pass and forward pass both write to the same depth buffer

2. **DirectX Backend** (`src/backends/directx/dx12_impl.rs:2327`):
   - Sets ONE render target and depth buffer for all passes via `OMSetRenderTargets`
   - All passes share these targets
   - Same problem: shadow pass and forward pass conflict on depth writes

### Manifestation of the Bug

When shadow mapping was enabled:
- Shadow pass rendered to the "main" depth buffer (should use shadow_map)
- Forward pass rendered to the same depth buffer  
- Result: Artifacts appeared (ghost geometry that didn't move with camera)
- The artifacts were the shadow pass's depth writes polluting the forward pass's depth buffer

## Correct Architecture

### Render Graph Modularity Principles

1. **Each pass is independent**: A pass should only access resources it declares as inputs/outputs
2. **Separate render targets**: Each pass should have its own framebuffer(s) based on its outputs
3. **Explicit dependencies**: Resource transitions between passes should be handled by barriers

### How It Should Work

#### Shadow Map Pass
```
Inputs:  vertex_buffer, index_buffer, light_uniforms  
Outputs: shadow_map (depth texture, 1024x1024, D32_FLOAT)

Execution:
1. Create/reuse framebuffer with shadow_map as depth attachment (no color attachment)
2. Begin render pass with this framebuffer
3. Render geometry from light's perspective
4. End render pass
5. Transition shadow_map from DepthStencilAttachment → ShaderReadOnly
```

#### Forward Pass
```
Inputs:  vertex_buffer, index_buffer, lighting_buffer, shadow_map (texture)
Outputs: swapchain_image (color), forward_pass_depth (depth)

Execution:
1. Create/reuse framebuffer with swapchain_image as color + forward_pass_depth as depth
2. Begin render pass with this framebuffer  
3. Bind shadow_map as texture sampler (binding 4)
4. Render geometry with shadows applied
5. End render pass
```

## Implementation Plan

### Phase 1: Per-Pass Framebuffer Management (Vulkan)

1. **Dynamic Framebuffer Creation**
   - In `execute_graph`, analyze each pass's outputs before execution
   - For each unique set of attachments, create a framebuffer
   - Cache framebuffers for reuse across frames

2. **Per-Pass Render Pass Creation**
   - Each pass declares its attachment formats (color, depth)
   - Create compatible render passes
   - Cache render passes for reuse

3. **Proper Barrier Insertion**
   - Insert barriers BETWEEN render passes (not during)
   - End current render pass before barrier
   - Insert barrier
   - Begin next render pass

### Phase 2: Per-Pass Render Targets (DirectX)

1. **Dynamic RTV/DSV Binding**
   - Before each pass, determine its output attachments
   - Get/create RTVs and DSVs for those resources
   - Call `OMSetRenderTargets` with pass-specific targets

2. **Resource State Tracking**
   - Track current state of each resource
   - Insert ResourceBarrier transitions as needed
   - Transition depth textures properly (DEPTH_WRITE ↔ SHADER_RESOURCE)

### Phase 3: Render Graph Resource Management

1. **Resource Lifetime Tracking**
   - Track which passes produce/consume each resource
   - Allocate resources before first use
   - Deallocate after last use (or keep alive for external access)

2. **Layout/State Inference**
   - Automatically determine optimal layouts from access patterns
   - Insert minimal barriers

## Current Workaround

Shadow mapping is temporarily disabled in `src/app.rs:243-257` until proper per-pass framebuffer support is implemented.

```rust
// Temporarily disabled - see docs/render_pass_modularity.md
let has_directional_light = false;
```

## Testing Plan

1. Re-enable shadow mapping
2. Verify shadow pass renders to separate 1024x1024 depth texture
3. Verify forward pass uses its own depth buffer
4. Verify no artifacts appear
5. Test on both Vulkan and DirectX backends

## Related Issues

- Issue #88: Index-driven rendering (completed)
- Issue #10: Shadow mapping (partially complete, awaiting this fix)
- New Issue: Per-pass framebuffer support (to be created)

## References

- `src/passes/shadow_map.rs`: Shadow map pass implementation
- `src/passes/forward_simple.rs`: Forward rendering pass
- `src/backends/vulkan/mod.rs:execute_graph`: Vulkan execution loop
- `src/backends/directx/dx12_impl.rs:execute_graph`: DirectX execution loop

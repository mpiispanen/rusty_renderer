# Render Graph Multi-Pass Architecture - Investigation Summary

## Date: 2025-11-06

## Investigation Overview

After implementing shadow mapping and camera controls, we discovered that having multiple render passes active (shadow + forward) produces visual artifacts. Investigation revealed a fundamental architectural issue with how backends execute the render graph.

## Key Findings

### ✅ What's Working (Render Graph Layer)

1. **Resource Declarations**: Correctly specified
   - `shadow_map` - Shadow pass depth output (1024x1024 Depth32Float)
   - `forward_pass_depth` - Forward pass depth buffer (1280x720 Depth32Float)
   - Separate resources, no sharing

2. **Dependency Tracking**: Working correctly
   - Shadow pass writes to `shadow_map`
   - Forward pass reads from `shadow_map` (as texture)
   - Forward pass writes to `forward_pass_depth` (as attachment)
   - Render graph identifies dependency chain

3. **Barrier Calculation**: Barriers are computed
   - Layout transitions calculated correctly
   - Access masks determined properly
   - Pipeline stages identified

4. **Pass Execution Order**: Topological sort working
   - Shadow pass executes before forward pass
   - No cyclic dependencies

### ❌ What's Broken (Backend Layer)

Both Vulkan and DX12 backends have the same fundamental issue:

**Single Render Pass Paradigm**
```rust
// Current (broken) execution in backends:
begin_render_pass(swapchain_framebuffer);  // ONE render pass
  for each_pass in render_graph {
    bind_pipeline(pass);
    execute_pass_callback();              // All draw to same targets!
  }
end_render_pass();
```

**Problems:**
1. All passes render to the SAME framebuffer
2. All passes share the SAME depth buffer  
3. No actual resource transitions between passes
4. Shadow map is overwritten by forward pass depth
5. Passes interfere with each other's state

### 📊 Visual Evidence

**Symptoms observed:**
- Fixed screen-space artifact shaped like the model
- Artifact has clear color (background)
- Artifact position doesn't change with camera movement
- Disabling depth pass removes artifact (proving depth sharing is the issue)

## Root Cause Analysis

### Backend Architecture Assumption

The backends were designed around a **single-pass rendering** model:
- One swapchain image
- One depth buffer
- One render pass per frame
- Perfect for simple forward rendering

### Render Graph Requirement

The render graph expects **true multi-pass** capability:
- Each pass has independent render targets
- Passes can read from previous passes' outputs
- Proper resource transitions between passes
- Modular, composable pipeline

### The Mismatch

When the render graph says "execute pass A then pass B":
- **Render Graph thinks**: "Pass A renders to its framebuffer, pass B renders to different framebuffer"
- **Backend actually does**: "Both passes render to the same swapchain framebuffer"

## Required Architecture Changes

### 1. Per-Pass Framebuffer Creation

Each pass needs its own framebuffer created from its output resources:

```rust
for pass in render_graph_passes {
    let framebuffer = create_framebuffer_from_outputs(pass.outputs);
    begin_render_pass(framebuffer);
    execute_pass(pass);
    end_render_pass();
}
```

### 2. Dynamic Render Pass Objects

Each unique set of attachments needs its own render pass object:
- Cache render passes by (color_formats, depth_format, sample_count)
- Create on-demand during graph execution
- Reuse across frames

### 3. Proper Resource Transitions

Insert actual barriers between passes:
```rust
end_render_pass(shadow_pass);
insert_barrier(shadow_depth: DepthAttachment → ShaderReadOnly);
begin_render_pass(forward_pass);
```

### 4. Render Target Management

- Swapchain images are special (final output)
- Intermediate passes use graph-allocated textures
- Each texture can be used as attachment OR sampled, not both simultaneously

## Implementation Strategy

### Phase 1: Proof of Concept (Vulkan)
- [ ] Implement per-pass framebuffer creation
- [ ] Create render pass objects dynamically
- [ ] Insert proper barriers between passes
- [ ] Test with shadow + forward passes

### Phase 2: Generalize
- [ ] Extract common multi-pass logic
- [ ] Create framebuffer/render pass cache
- [ ] Handle all resource transition cases

### Phase 3: Port to DX12
- [ ] Adapt DX12 backend to multi-pass model
- [ ] Verify parity with Vulkan
- [ ] Performance optimization

### Phase 4: Advanced Features
- [ ] Multi-pass effects (bloom, SSAO, etc.)
- [ ] Render pass graph visualization
- [ ] Resource lifetime optimization

## Files Modified

### Code Changes
- `src/app.rs` - Renamed depth buffer to `forward_pass_depth`, uncommented shadow pass
- `RENDERPASS_ARCHITECTURE_FIX.md` - Created comprehensive fix document

### GitHub Issues
- **Created Issue #94**: "Fix render graph multi-pass architecture"  
- **Updated Issue #90**: Added architecture blocker comment
- Shadow mapping implementation blocked on #94

## Testing Plan

Once #94 is implemented:

1. **Visual Verification**
   - No artifacts with shadow mapping enabled
   - Shadows correctly cast on geometry
   - Camera movement doesn't cause glitches

2. **Technical Verification**
   - Separate framebuffers for each pass (validation layers)
   - Proper barrier insertion (check logs)
   - Correct resource layouts at each pass boundary

3. **Multi-Backend Verification**
   - Identical visual output on Vulkan and DX12
   - Performance comparison
   - Validation of resource state tracking

## Lessons Learned

1. **Architecture Assumptions**: Early design decisions have far-reaching effects
2. **Layer Boundaries**: Clear contracts between graph and backend are crucial
3. **Progressive Development**: Simple cases (single pass) hid the architecture issue
4. **Testing Importance**: Multi-pass scenarios revealed the design flaw

## Next Steps

1. Implement Issue #94 (multi-pass backend architecture)
2. Re-enable shadow sampling in forward shader
3. Test shadow quality and adjust parameters
4. Implement additional multi-pass effects

## References

- Issue #90: Shadow mapping implementation
- Issue #94: Multi-pass architecture fix
- `RENDERPASS_ARCHITECTURE_FIX.md`: Detailed implementation guide
- Vulkan backend: `src/backends/vulkan/mod.rs:3102` (execute_graph)
- DX12 backend: Similar issues exist

---

**Status**: Architecture issue identified and documented. Implementation of fix tracked in #94.

**Impact**: Blocks all multi-pass rendering techniques (shadows, post-processing, deferred rendering).

**Priority**: High - Core architecture issue affecting multiple features.

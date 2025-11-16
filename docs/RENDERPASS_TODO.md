# Render Pass Architecture TODO

## Overview

Currently, some rendering configuration is hardcoded in the backend implementations rather than being driven by render pass definitions. This document tracks what needs to be moved to the render graph/pass definitions.

## Current State

The forward rendering pass (`ForwardSimplePass`) and shadow mapping pass (`ShadowMapPass`) are mostly integrated with the render graph, but some aspects are still hardcoded in backend implementations.

## Items to Move to Render Pass Definitions

### 1. Clear Colors ✅ (Partially Done, Needs Completion)

**Current**: Clear colors are hardcoded in backends:
- Vulkan: `[0.0, 0.0, 0.0, 1.0]` (black) in `src/backends/vulkan/mod.rs:4903`
- DirectX: `[0.0, 0.0, 0.0, 1.0]` (black) in `src/backends/directx/dx12_impl.rs:2142`

**Should be**: Defined in render pass configuration
- Each render pass should specify clear values for its attachments
- Different passes might want different clear colors (e.g., shadow maps, post-processing)

**Action Items**:
1. Add `clear_value: Option<[f32; 4]>` to `ResourceAccess` or attachment descriptor
2. Update `RenderPass` to include attachment clear values
3. Pass clear values from render pass to backend during `begin_render_pass()`
4. Remove hardcoded clear colors from backend implementations

**Example**:
```rust
// In render pass definition
let color_attachment = AttachmentDescriptor {
    resource: color_target,
    layout: ImageLayout::ColorAttachment,
    clear_value: Some([0.1, 0.1, 0.15, 1.0]), // Dark blue-gray
    load_op: LoadOp::Clear,
    store_op: StoreOp::Store,
};
```

### 2. Depth Clear Values

**Current**: Depth buffers cleared to 1.0 (far plane) by default, hardcoded in backends

**Should be**: Specified per render pass
- Shadow maps might need different depth clear values
- Some passes might not want to clear depth at all

**Action Items**:
1. Add depth clear value to attachment descriptors
2. Support `LoadOp::Clear` vs `LoadOp::Load` for depth attachments
3. Pass depth clear values from render pass to backend

### 3. Attachment Load/Store Operations

**Current**: Implicitly handled - always clear and store

**Should be**: Explicitly specified in render pass
- `LoadOp::Clear` - Clear attachment at start
- `LoadOp::Load` - Preserve existing content
- `LoadOp::DontCare` - Don't care about initial content
- `StoreOp::Store` - Save result to memory
- `StoreOp::DontCare` - Don't need to save result

**Benefit**: 
- Performance optimization (avoid unnecessary clears/stores)
- Explicit intent (clear vs preserve)
- Multi-pass rendering (load result from previous pass)

**Action Items**:
1. Add `LoadOp` and `StoreOp` enums to render graph
2. Add to attachment descriptors
3. Implement in backends (especially important for Vulkan tile-based GPUs)

### 4. Shader Bindings and Descriptor Sets

**Current**: Binding locations partially hardcoded in backends
- Forward pass expects: binding 0 = camera, binding 1 = lighting, binding 2 = shadow, binding 3+ = textures
- Defined in shader code and matched in pass implementation

**Should be**: Defined by render pass, validated against shader reflection
- Render pass specifies what resources bind where
- Backend creates appropriate descriptor sets based on pass requirements
- Shader reflection validates compatibility

**Status**: Mostly handled through render graph resource system, but could be more explicit

**Future Improvement**:
- Add shader reflection to validate bindings
- Auto-generate binding layouts from shader SPIR-V
- Detect mismatches between render pass and shader expectations

### 5. Pipeline Configuration

**Current**: Some pipeline state hardcoded in shader creation
- Depth test enable/disable
- Blend modes
- Cull modes
- Polygon modes

**Should be**: Specified in render pass or pipeline descriptor
- Each pass defines its required pipeline state
- Multiple passes can share shaders but with different pipeline configs

**Partially Done**: `PipelineBuilder` exists but not fully utilized

**Action Items**:
1. Expand `PipelineBuilder` with more state options
2. Have render passes specify complete pipeline state
3. Remove hardcoded pipeline state from backend shader creation

### 6. Viewport and Scissor

**Current**: Set to full framebuffer size automatically

**Should be**: Optionally specified by render pass
- Most passes want full viewport
- Some passes might want sub-regions (e.g., cascaded shadow maps)

**Priority**: Low (current default is usually correct)

### 7. Subpass Dependencies

**Current**: Each pass is independent

**Should be**: Support subpass dependencies for optimization
- Multiple passes can be grouped into subpasses
- Reduces memory bandwidth on tile-based GPUs
- Explicit synchronization between subpasses

**Priority**: Low (optimization, not correctness)

## Architecture Goals

### Separation of Concerns

1. **Render Pass**: Defines WHAT to render and HOW
   - Input/output resources
   - Attachments and their formats
   - Clear values
   - Load/store operations
   - Pipeline state requirements
   - Resource bindings

2. **Render Graph**: Manages resource lifetimes and ordering
   - Allocates resources
   - Determines execution order
   - Inserts synchronization
   - Optimizes resource usage

3. **Backend**: Implements the actual rendering
   - Receives fully-specified render pass
   - Creates backend-specific objects (pipelines, descriptor sets)
   - Executes rendering commands
   - NO hardcoded rendering logic

### Application Code

The application (`app.rs`) should:
- Load scenes and resources
- Build render graph with passes
- NOT contain rendering logic
- NOT know about backends
- NOT hardcode bindings or formats

Currently, `app.rs` has some special cases that should be moved:
- Shadow map creation (should be in shadow pass)
- Default texture creation (should be in material system)
- Resource binding decisions (should be in render passes)

## Implementation Priority

### High Priority
1. ✅ Clear colors in render pass definitions
2. Depth clear values
3. Load/store operations

### Medium Priority
4. Complete pipeline configuration in passes
5. Remove remaining hardcoded bindings from app.rs
6. Shader reflection for validation

### Low Priority
7. Viewport/scissor per-pass
8. Subpass dependencies
9. Advanced optimization (tile-based rendering)

## Testing

After each change:
1. Verify Vulkan backend still renders correctly
2. Verify DirectX backend still renders correctly
3. Compare outputs to golden references
4. Check that CI passes

## Benefits of This Architecture

1. **Flexibility**: Easy to add new render passes
2. **Maintainability**: Clear separation of concerns
3. **Portability**: Backend-agnostic pass definitions
4. **Optimization**: Graph can optimize resource usage and barriers
5. **Debugging**: Clear flow from pass definition to execution

## Current Pass Examples

### Forward Pass
- Inputs: vertex buffer, camera uniforms, lighting uniforms, textures
- Outputs: color target, depth target
- Clear: Should specify clear color (currently hardcoded to black)
- Pipeline: 3D rendering with depth test

### Shadow Map Pass
- Inputs: vertex buffer, shadow camera uniforms
- Outputs: shadow map (depth only)
- Clear: Depth to 1.0
- Pipeline: Depth-only rendering

## Next Steps

1. Add clear value support to render pass attachments
2. Update forward pass to specify dark gray-blue clear color
3. Update shadow pass to specify depth clear
4. Test both backends with new clear values
5. Move on to load/store operations
6. Continue down the priority list

## See Also

- `src/render_graph/pass.rs` - Pass definitions
- `src/render_graph/graph.rs` - Render graph implementation
- `src/passes/forward_simple.rs` - Forward pass implementation
- `src/passes/shadow_map.rs` - Shadow pass implementation

# Forward Pass Migration Session - 2025-10-30

## Objective
Migrate forward rendering pass to use render graph resource management instead of hardcoded paths and manual buffer passing.

## What We Accomplished

### 1. Created ForwardSimplePass ✅
- **Location**: `src/passes/forward_simple.rs`
- **Purpose**: Demonstrates the target architecture for render passes
- **Key Features**:
  - Uses only `ResourceId` for all resources (no direct Buffer/Texture passing)
  - All resources managed by render graph
  - Declarative dependencies (inputs/outputs clearly defined)
  - Builder pattern for easy configuration
  - Proper resource dependency tracking

### 2. Architecture Pattern
```rust
// Create resources in graph
let vertex_buffer = graph.create_resource("vertices", ResourceDescriptor::Buffer { ... });
let camera_buffer = graph.create_resource("camera", ResourceDescriptor::Buffer { ... });

// Create pass with resource IDs
let pass = ForwardSimplePass::builder()
    .color_output(color_res)
    .depth_output(depth_res)
    .vertex_buffer(vertex_buffer)
    .camera_buffer(camera_buffer)
    .lighting_buffer(lighting_buffer)
    .vertex_count(36)
    .build(&mut graph)?;
```

### 3. Benefits of New Approach
1. **No Hardcoded Paths**: Resources referenced by ResourceId
2. **Centralized Management**: Graph allocates and tracks all resources
3. **Automatic Dependencies**: Graph knows what each pass reads/writes
4. **Cleaner API**: Builder pattern makes configuration clear
5. **Type Safety**: Compile-time checks for required resources

## Current State

### Pass Types
1. **TrianglePass**: Simple debug pass (hardcoded triangle in shader)
2. **VertexBufferTrianglePass**: Triangle with vertex buffer (transition state)
3. **ForwardDeclarativePass**: Forward pass with DeclarativePass trait (transition state)
4. **ForwardSimplePass**: NEW - target architecture with full graph integration

### What's Next

#### Phase 1: Resource Upload (Immediate)
Currently, buffers need a "producer" pass to be valid in the graph. We need to:
- Add support for "external" or "imported" resources (CPU-uploaded buffers)
- OR create Transfer passes that populate buffers from CPU data
- This will allow full graph compilation with buffer resources

#### Phase 2: Full Execution Implementation
Implement the `execute()` callback in ForwardSimplePass:
- Access resources from execution context
- Bind pipeline
- Push constants (model + normal matrices)
- Bind descriptor sets
- Bind vertex buffer
- Draw call

#### Phase 3: Example Integration
Update examples to use ForwardSimplePass:
- Modify `examples/gltf_viewer.rs` to create resources in graph
- Demonstrate the new pattern in action
- Verify rendering works correctly

#### Phase 4: Deprecation
Once ForwardSimplePass is fully working:
- Deprecate ForwardDeclarativePass
- Update all examples to use new pattern
- Remove old transition code

## Technical Details

### Resource Declaration
```rust
// Outputs
pass.add_output(ResourceAccess::new(
    color_output,
    AccessType::Write,
    PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
    Some(ImageLayout::ColorAttachment),
));

// Inputs
pass.add_input(ResourceAccess::new(
    vertex_buffer,
    AccessType::Read,
    PipelineStage::new(PipelineStage::VERTEX_INPUT),
    None,
));
```

### Pipeline Declaration
```rust
impl PassCallback for ForwardSimplePassCallback {
    fn declare_pipeline(&self, builder: &mut PipelineBuilder, registry: &ShaderRegistry) {
        let vs = registry.get_handle("forward.vert")?;
        let fs = registry.get_handle("forward.frag")?;
        
        builder
            .vertex_shader(vs)
            .fragment_shader(fs)
            .depth_test(true)
            .depth_write(true)
            .cull_mode(CullMode::Back);
    }
}
```

## Files Changed
- `src/passes/forward_simple.rs` - NEW: Target architecture pass
- `src/passes/mod.rs` - Export new pass

## Tests
✅ All 127 library tests passing
✅ ForwardSimplePass builder test
✅ ForwardSimplePass validation test (missing resources)

## Next Session Goals
1. Add external/imported resource support to render graph
2. Implement full execute() callback for ForwardSimplePass
3. Create an example demonstrating the new pattern
4. Update documentation with migration guide

## Notes
- This is the TARGET pattern we want all passes to follow
- Current ForwardDeclarativePass is a transition state
- Once we have external resource support, we can compile full graphs
- The new pattern significantly improves code clarity and maintainability

---
*Status: Phase 1 Complete - Basic structure implemented*
*Next: Resource import/upload system*

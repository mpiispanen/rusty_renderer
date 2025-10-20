# M9: Render Graph Integration - Proper Pass Execution

**Status:** Planning  
**Priority:** High  
**Goal:** Make render graph actually execute passes through proper command recording, not workarounds

---

## Problem Statement

Currently, our render graph infrastructure is complete but **not actually used for rendering**:

- ✅ Render graph API exists (graph building, pass definition, compilation)
- ✅ Dependency resolution and barrier insertion works
- ✅ Execution order is computed correctly
- ❌ **But passes don't actually execute through the render graph**
- ❌ Examples use backend directly or workarounds (raw pointers)
- ❌ PassExecutionContext is a stub interface

**What we have:**
```rust
// This compiles and validates the graph but doesn't actually render
let compiled = graph.compile()?;
backend.execute_graph(&graph, &compiled)?; // Stub implementation!
```

**What we need:**
```rust
// This should actually execute the pass callbacks and record commands
let compiled = graph.compile()?;
backend.execute_graph(&graph, &compiled)?; // Properly implemented!
  // → Calls pass.execute(context) for each pass
  //   → Context records actual GPU commands
  //   → Barriers are inserted
  //   → Resources are transitioned
```

## Current Issues

### Issue #41: Render graph not connected to actual rendering
The render graph is validated and working but doesn't drive actual rendering.

### Issue #51: Vertex buffer example bypasses render graph  
Uses raw pointers and workarounds instead of proper pass execution.

### Issue #53: Texture loading works but can't be used in render passes
We can load textures but have no way to actually render with them through the graph.

## Goals

### Primary Goal
**Make render graph execute passes properly through PassExecutionContext**

This means:
1. Backend implements `execute_graph()` to iterate through compiled passes
2. For each pass, creates a proper execution context
3. Calls `pass.execute(context)` with a context that records real commands
4. Command buffer is submitted after all passes

### Secondary Goals
1. Create **proper render pass implementations** (not workarounds)
2. Demonstrate with **two working examples:**
   - Triangle pass (simplest case)
   - Textured quad pass (with texture binding)
3. Validate output matches expectations
4. Clean up workarounds from M8.2

## Architecture

### Current Flow (Broken)
```
RenderGraph::compile()
  → CompiledGraph with execution order
    → backend.execute_graph()  
      → STUB! Does nothing or hardcoded rendering
```

### Desired Flow
```
RenderGraph::compile()
  → CompiledGraph with execution order
    → backend.execute_graph()
      → For each pass in execution_order:
          → Begin render pass (framebuffer, clear)
          → Create PassExecutionContext (wraps command buffer)
          → pass_callback.execute(context)
            → Context records: bind pipeline, bind resources, draw
          → End render pass
          → Insert barriers (if needed)
      → Submit command buffer
```

### PassExecutionContext Implementation

Each backend needs to implement:

```rust
struct VulkanPassContext<'a> {
    command_buffer: vk::CommandBuffer,
    device: &'a Device,
    pipeline: vk::Pipeline,  // From pass metadata
    descriptor_sets: Vec<vk::DescriptorSet>,  // From bound resources
}

impl PassExecutionContext for VulkanPassContext<'_> {
    fn draw(&mut self, vertex_count: u32, instance_count: u32, ...) {
        // Actually record vkCmdDraw!
        unsafe {
            self.device.cmd_draw(
                self.command_buffer,
                vertex_count,
                instance_count,
                ...
            );
        }
    }
    
    fn bind_vertex_buffer(&mut self, binding: u32, buffer_ptr, offset) {
        // Downcast buffer_ptr to VulkanBuffer
        // Actually record vkCmdBindVertexBuffers!
    }
    
    // ... implement all PassExecutionContext methods
}
```

## Implementation Phases

### Phase 1: Backend Execute Graph Implementation (6-8 hours)

**Vulkan:**
- Implement `execute_graph()` to iterate passes
- For each pass:
  - Begin render pass with proper attachments
  - Create `VulkanPassContext` with command buffer
  - Call `pass.execute(context)`
  - End render pass
- Handle barriers between passes
- Submit command buffer

**wgpu:**
- Similar implementation for wgpu
- Use render pass encoder as context

**DirectX:**
- Can defer or implement stub for now

**Key Changes:**
```rust
// src/backends/vulkan/mod.rs
impl GraphicsBackend for VulkanBackend {
    fn execute_graph(&mut self, graph: &RenderGraph, compiled: &CompiledGraph) -> Result<()> {
        let cmd_buffer = self.current_command_buffer?;
        
        for pass_id in &compiled.execution_order {
            let pass = graph.get_pass(*pass_id)?;
            
            // Begin render pass
            self.begin_render_pass(pass, cmd_buffer)?;
            
            // Create execution context
            let mut context = VulkanPassContext::new(
                cmd_buffer,
                &self.device,
                self.get_pipeline_for_pass(pass)?,
            );
            
            // Execute the pass callback
            pass.callback().execute(&mut context);
            
            // End render pass
            self.end_render_pass(cmd_buffer)?;
            
            // Insert barriers if needed
            self.insert_barriers(&compiled.barriers[pass_id])?;
        }
        
        Ok(())
    }
}
```

### Phase 2: Proper Pass Implementations (4-6 hours)

**TrianglePass:**
```rust
struct TrianglePass {
    // No workarounds! Just pass data
    vertex_buffer: Arc<dyn Buffer>,
    // Could also reference by resource ID in graph
}

impl PassCallback for TrianglePass {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        // Bind pipeline (context knows which one from pass setup)
        
        // Bind vertex buffer properly
        context.bind_vertex_buffer(0, &*self.vertex_buffer)?;
        
        // Draw
        context.draw(3, 1, 0, 0)?;
        
        // No raw pointers, no unsafe, no workarounds!
    }
}
```

**TexturedQuadPass:**
```rust
struct TexturedQuadPass {
    vertex_buffer: Arc<dyn Buffer>,
    index_buffer: Arc<dyn Buffer>,
    texture: Arc<dyn Texture>,
    sampler: Arc<dyn Sampler>,
}

impl PassCallback for TexturedQuadPass {
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        // Bind pipeline
        
        // Bind vertex/index buffers
        context.bind_vertex_buffer(0, &*self.vertex_buffer)?;
        context.bind_index_buffer(&*self.index_buffer)?;
        
        // Bind texture/sampler through descriptor set
        context.bind_descriptor_set(0, &[
            BoundResource::Texture(self.texture.clone()),
            BoundResource::Sampler(self.sampler.clone()),
        ])?;
        
        // Draw indexed
        context.draw_indexed(6, 1, 0, 0, 0)?;
    }
}
```

### Phase 3: Examples and Validation (2-3 hours)

**triangle_render_graph.rs:**
- Create triangle pass properly
- Add to render graph
- Execute through graph
- Capture and save output

**textured_quad_render_graph.rs:**
- Create textured quad pass
- Load texture and create GPU resources
- Add to render graph  
- Execute through graph
- Capture and save output

**Validation:**
- Visual comparison with reference images
- Screenshots should match expected output
- Performance should be reasonable

### Phase 4: Cleanup and Documentation (1-2 hours)

- Remove workarounds from M8.2 examples
- Update documentation
- Add architecture diagram showing flow
- Close related issues

## API Design

### PassExecutionContext Additions

Need to add descriptor set binding:

```rust
pub trait PassExecutionContext {
    // ... existing methods ...
    
    /// Bind a descriptor set (M9)
    fn bind_descriptor_set(
        &mut self,
        set: u32,
        resources: &[BoundResource],
    ) -> anyhow::Result<()>;
    
    /// Bind a pipeline (M9) 
    fn bind_pipeline(&mut self, pipeline_handle: usize) -> anyhow::Result<()>;
}
```

### RenderPass Metadata

Passes need to declare their pipeline requirements:

```rust
pub struct RenderPass {
    // ... existing fields ...
    
    /// Pipeline configuration for this pass
    pub pipeline_config: PipelineConfig,
}

pub struct PipelineConfig {
    pub vertex_shader: ShaderHandle,
    pub fragment_shader: ShaderHandle,
    pub vertex_layout: VertexBufferLayout,
    pub descriptor_layouts: Vec<BindGroupLayout>,
}
```

## Success Criteria

### Must Have
- ✅ `execute_graph()` properly implemented for Vulkan
- ✅ `execute_graph()` properly implemented for wgpu  
- ✅ PassExecutionContext fully functional (not stub)
- ✅ Triangle renders through render graph (no workarounds)
- ✅ Textured quad renders through render graph (no workarounds)
- ✅ Visual output matches expectations
- ✅ No raw pointers or unsafe workarounds in examples
- ✅ All tests pass

### Should Have
- ✅ Performance within 5% of direct rendering
- ✅ Clean separation between graph and execution
- ✅ Good error messages
- ✅ Example documentation

### Nice to Have
- Visual regression testing with FLIP
- DirectX implementation
- Performance benchmarks

## Dependencies

### Required
- Render graph core (already complete)
- Vertex buffers (M8.2 - complete)
- Textures (M8.4 - complete)
- Shader resource binding (M8.3 - complete)

### Blocks
- Issue #41 (render graph refactor)
- Issue #51 (vertex buffer workarounds)
- All future rendering work needs this

## Risks

### Medium Risk: Complexity
- Backend command recording can be tricky
- Need to handle state management properly
- Pipeline binding needs careful design

**Mitigation:** Start with simple case (triangle), then add complexity

### Low Risk: Performance
- Extra abstraction might add overhead
- But should be minimal (function call overhead only)

**Mitigation:** Profile and optimize if needed

## Timeline Estimate

**Total: 13-19 hours**

- Phase 1: Backend implementation (6-8 hours)
- Phase 2: Pass implementations (4-6 hours)
- Phase 3: Examples/validation (2-3 hours)
- Phase 4: Cleanup/docs (1-2 hours)

Can be split across multiple sessions.

## Out of Scope

- Complex multi-pass rendering
- Compute passes
- Async compute
- Multiple render targets
- Post-processing
- Full forward renderer (that's next milestone)

## Next Steps After M9

With proper render graph execution, we can then build:

**M10: Forward Renderer Foundation**
- Camera system
- Transform/MVP matrices
- Lighting (single directional light)
- Render a lit, textured mesh
- Foundation for glTF rendering

**M11: glTF Model Loading**
- Load glTF files
- Parse meshes, materials, textures
- Build scene graph
- Render loaded models

## Notes

This milestone is **critical** - it's the foundation for all future rendering work. We've been building infrastructure (render graph, resources, shaders) but haven't proven it actually works together. This milestone ties it all together properly.

After this, we'll have:
- **Working render graph** that actually executes
- **Clean pass implementations** as templates
- **Foundation for complex rendering** (forward renderer, deferred, etc.)
- **No more workarounds** - proper architecture throughout

## References

- Issue #41: Render graph refactor
- Issue #51: Vertex buffer rendering
- Issue #53: Texture loading
- `docs/M6_PLANNING.md`: Render graph design
- `src/render_graph/`: Current implementation
- `examples/vertex_buffer_triangle.rs`: Current workaround example

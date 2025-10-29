# Declarative Pipeline Compilation Design

## Current State

We have:
- ✅ ShaderRegistry integrated with RenderGraph
- ✅ Shaders registered during graph build
- ✅ ForwardDeclarativePass with `declare_pipeline()` method
- ✅ PipelineBuilder that collects pipeline state
- ❌ No mechanism to compile pipeline declarations into backend pipelines
- ❌ Backends still use hardcoded pipelines

## Problem

The declarative system is incomplete:

1. **Pipeline Declarations Not Collected**: `declare_pipeline()` is never called
2. **No Pipeline Compilation**: PipelineBuilder state isn't converted to backend pipelines
3. **Hardcoded Execution**: Backends bind hardcoded pipelines regardless of declarations

## Solution Design

### Phase 1: Pipeline Compilation in Graph

**Goal**: Compile pipeline declarations into backend-specific pipeline objects during graph compilation.

#### Changes to RenderGraph::compile()

```rust
pub fn compile(&mut self, backend: &mut dyn GraphicsBackend) -> Result<CompiledGraph> {
    // Existing: dependency analysis, topological sort
    let execution_order = self.topological_sort()?;
    let producers = self.find_producers()?;
    let barriers = self.insert_barriers(&execution_order, &producers)?;
    
    // NEW: Compile pipelines for each pass
    let mut pipelines = HashMap::new();
    
    for &pass_id in &execution_order {
        if let Some(pass) = self.get_pass(pass_id) {
            if pass.kind == PassKind::Graphics {
                // Build pipeline for this pass
                let pipeline = self.compile_pipeline_for_pass(pass, backend)?;
                pipelines.insert(pass_id, pipeline);
            }
        }
    }
    
    Ok(CompiledGraph {
        execution_order,
        producers,
        barriers,
        pipelines,  // NEW
    })
}
```

#### New Method: compile_pipeline_for_pass()

```rust
fn compile_pipeline_for_pass(
    &self,
    pass: &RenderPass,
    backend: &mut dyn GraphicsBackend,
) -> Result<PipelineHandle> {
    // Create pipeline builder
    let mut builder = PipelineBuilder::new();
    
    // Let the pass declare its pipeline requirements
    if let Some(callback) = &pass.callback {
        // TODO: Need way to call declare_pipeline through callback
        // For now, we need to extract the DeclarativePass from the adapter
    }
    
    // Get shader modules from registry
    let shader_handles = builder.shaders();
    let mut shader_modules = Vec::new();
    
    for handle in shader_handles {
        let descriptor = self.shader_registry.get_by_handle(*handle)?;
        let module = backend.create_shader_module(descriptor)?;
        shader_modules.push(module);
    }
    
    // Create pipeline from builder state
    let pipeline = backend.create_graphics_pipeline(
        &shader_modules,
        builder.vertex_layout(),
        builder.depth_state(),
        builder.rasterizer_state(),
        builder.blend_states(),
    )?;
    
    Ok(pipeline)
}
```

### Phase 2: Backend Shader Compilation

**Goal**: Backends can create shader modules from descriptors in the registry.

#### New GraphicsBackend Method

```rust
trait GraphicsBackend {
    // ... existing methods ...
    
    /// Create a shader module from a shader descriptor
    ///
    /// The descriptor may contain:
    /// - Embedded bytecode (SPIR-V/DXIL) - use directly
    /// - File path - load and compile
    /// - Inline source - compile
    fn create_shader_module(
        &mut self,
        descriptor: &ShaderDescriptor,
    ) -> Result<ShaderModuleHandle>;
}
```

#### Vulkan Implementation

```rust
impl GraphicsBackend for VulkanBackend {
    fn create_shader_module(
        &mut self,
        descriptor: &ShaderDescriptor,
    ) -> Result<ShaderModuleHandle> {
        let bytecode = match &descriptor.source {
            ShaderSource::Embedded(bytes) => {
                // Convert &[u8] to &[u32] for SPIR-V
                bytes_to_u32_vec(bytes)
            }
            ShaderSource::Compiled(path) => {
                // Load pre-compiled SPIR-V
                let bytes = std::fs::read(path)?;
                bytes_to_u32_vec(&bytes)
            }
            ShaderSource::File(path) => {
                // Compile HLSL to SPIR-V using DXC
                compile_hlsl_to_spirv(path, descriptor.entry_point, descriptor.stage)?
            }
        };
        
        let create_info = vk::ShaderModuleCreateInfo::builder()
            .code(&bytecode);
        
        let module = unsafe {
            self.device.create_shader_module(&create_info, None)?
        };
        
        Ok(ShaderModuleHandle::Vulkan(module))
    }
}
```

### Phase 3: Execution with Declarative Pipelines

**Goal**: Execute graph using pipelines from declarations, not hardcoded.

#### Changes to execute_graph()

```rust
fn execute_graph(
    &mut self,
    graph: &RenderGraph,
    compiled: &CompiledGraph,
) -> Result<()> {
    // ... begin frame, render pass ...
    
    for &pass_id in &compiled.execution_order {
        let pass = graph.get_pass(pass_id).unwrap();
        
        // Bind the pipeline for this pass
        if let Some(pipeline) = compiled.pipelines.get(&pass_id) {
            self.bind_pipeline(pipeline);
        }
        
        // Execute the pass
        if let Some(callback) = &pass.callback {
            callback.execute(&mut context);
        }
    }
    
    // ... end render pass, frame ...
}
```

## Challenges

### Challenge 1: Accessing DeclarativePass from Adapter

The `DeclarativePassAdapter` wraps the pass, but we need to call `declare_pipeline()`.

**Solution**: Add method to PassCallback:

```rust
trait PassCallback {
    fn prepare(&self, context: &mut dyn PassPreparationContext);
    fn execute(&self, context: &mut dyn PassExecutionContext);
    
    // NEW
    fn declare_pipeline(
        &self,
        builder: &mut PipelineBuilder,
        registry: &ShaderRegistry,
    ) {
        // Default: no pipeline
    }
}
```

Then implement in DeclarativePassAdapter:

```rust
impl<T: DeclarativePass> PassCallback for DeclarativePassAdapter<T> {
    fn declare_pipeline(
        &self,
        builder: &mut PipelineBuilder,
        registry: &ShaderRegistry,
    ) {
        self.pass.declare_pipeline(builder, registry);
    }
}
```

### Challenge 2: Backend-Specific Pipeline Handles

Each backend has different pipeline types:
- Vulkan: `vk::Pipeline`
- DirectX: `ID3D12PipelineState`

**Solution**: Use an enum:

```rust
pub enum PipelineHandle {
    Vulkan(vk::Pipeline),
    DirectX(ComPtr<ID3D12PipelineState>),
}
```

Or use trait objects:

```rust
pub trait Pipeline: Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

// In CompiledGraph:
pipelines: HashMap<PassId, Box<dyn Pipeline>>,
```

### Challenge 3: Pipeline Creation Complexity

Creating pipelines requires more than just shaders:
- Render pass compatibility
- Vertex layouts
- Descriptor set layouts
- Push constant ranges

**Solution**: Phase it in:

1. **Phase 3a**: Simple graphics pipeline (hardcoded layouts)
2. **Phase 3b**: Declarative vertex layouts
3. **Phase 3c**: Declarative descriptor layouts
4. **Phase 3d**: Full pipeline customization

## Implementation Plan

### Step 1: Add declare_pipeline to PassCallback ✅ (Already exists!)

Actually, reviewing the code, `DeclarativePass` already has `declare_pipeline()`. We just need to call it!

### Step 2: Collect Pipeline Descriptions

Add to RenderGraph:

```rust
pub struct PipelineDescription {
    pub shaders: Vec<ShaderHandle>,
    pub vertex_layout: Option<VertexLayout>,
    pub depth_state: DepthState,
    pub rasterizer_state: RasterizerState,
    pub blend_states: Vec<BlendState>,
}

impl RenderGraph {
    pub fn get_pipeline_description(&self, pass_id: PassId) -> Result<PipelineDescription> {
        // ... call declare_pipeline and build description ...
    }
}
```

### Step 3: Backend Shader Module Creation

Implement `create_shader_module` in each backend (start with Vulkan).

### Step 4: Simple Pipeline Compilation

Create pipelines with fixed layouts first, full declarative later.

### Step 5: Update Execution

Use per-pass pipelines instead of single hardcoded pipeline.

## Testing Strategy

1. **Unit Tests**: Test pipeline description building
2. **Integration Tests**: Test shader module creation from registry
3. **Rendering Tests**: Verify output matches with declarative pipelines
4. **Backend Parity**: Ensure Vulkan and DirectX produce same results

## Benefits

Once complete:

1. **No Hardcoded Pipelines**: All pipelines declared by passes
2. **Reusable Shaders**: Same shader used across multiple pipelines
3. **Hot Reload Ready**: Infrastructure for runtime recompilation
4. **Testable**: Pipeline creation testable independently
5. **Flexible**: Easy to add new passes and pipeline variants

## Timeline

- **Week 1**: PassCallback updates, pipeline description collection
- **Week 2**: Backend shader module creation (Vulkan)
- **Week 3**: Simple pipeline compilation and execution
- **Week 4**: Full declarative pipeline with layouts
- **Week 5**: DirectX backend support, testing

## Next Immediate Steps

1. Add `declare_pipeline` to PassCallback trait
2. Implement in DeclarativePassAdapter
3. Add `get_pipeline_description()` to RenderGraph
4. Test pipeline description building

---

*Created: 2025-10-29*
*Status: Design Phase*

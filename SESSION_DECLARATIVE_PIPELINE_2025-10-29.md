# Declarative Pipeline Infrastructure Session - 2025-10-29

## Summary

Completed major infrastructure for the declarative pipeline system, enabling render graph to collect and manage pipeline requirements from passes.

## Accomplishments

### 1. Shader Registry Integration ✅

**Files Modified:**
- `src/render_graph/graph.rs` - Added ShaderRegistry
- `src/pipelines/forward.rs` - Registered shaders
- `src/passes/forward_declarative.rs` - Uses registry

**What We Did:**
- Integrated ShaderRegistry into RenderGraph
- Added `register_shader()` and accessor methods
- Registered forward pass shaders using embedded SPIR-V
- ForwardDeclarativePass retrieves shaders from registry

### 2. Pipeline Description Collection ✅

**Files Modified:**
- `src/render_graph/pass.rs` - Extended PassCallback
- `src/render_graph/graph.rs` - Added get_pipeline_description()
- `src/render_graph/pipeline.rs` - Added accessors

**What We Did:**
- Added `declare_pipeline()` to PassCallback trait
- Implemented in DeclarativePassAdapter
- Added `get_pipeline_description()` to RenderGraph
- Added getter methods to PipelineBuilder

### 3. Design Documentation ✅

**Files Created:**
- `DECLARATIVE_PIPELINE_DESIGN.md` - Complete design doc
- `SESSION_SHADER_REGISTRY_2025-10-29.md` - Session notes

**What We Did:**
- Documented full architecture for declarative pipelines
- Outlined implementation plan
- Identified challenges and solutions

## Technical Details

### Shader Registration Flow

```rust
// In ForwardPipeline::build_graph():
graph.register_shader(
    "forward.vert",
    ShaderDescriptor {
        source: ShaderSource::Embedded(include_bytes!("../../shaders/forward.vert.spv")),
        entry_point: "VSMain",
        stage: ShaderStage::Vertex,
        backend_compile: false,
    },
);
```

### Pipeline Description Collection

```rust
// Get pipeline requirements from a pass:
let builder = graph.get_pipeline_description(pass_id)?;

// Access pipeline state:
let shaders = builder.shaders();
let depth_state = builder.get_depth_state();
let vertex_layout = builder.get_vertex_layout();
```

### Declaration Flow

```
Pass::declare_pipeline(builder, registry)
  ├─> registry.get_handle("forward.vert")
  ├─> builder.vertex_shader(handle)
  ├─> builder.fragment_shader(handle)
  ├─> builder.depth_test(true)
  └─> builder.cull_mode(CullMode::Back)

RenderGraph::get_pipeline_description(pass_id)
  ├─> Creates PipelineBuilder
  ├─> Calls pass.callback.declare_pipeline()
  └─> Returns PipelineBuilder with requirements
```

## Architecture

### Current State

```
Application
  └─> ForwardPipeline::build_graph()
      ├─> RenderGraph::new()
      │   └─> ShaderRegistry::new()
      │
      ├─> graph.register_shader("forward.vert", ...)
      └─> graph.register_shader("forward.frag", ...)

Pass Creation
  └─> ForwardDeclarativePass::new(...)
      └─> graph.add_declarative_pass(pass)
          └─> Creates DeclarativePassAdapter

Pipeline Description (NEW)
  └─> graph.get_pipeline_description(pass_id)
      ├─> Creates PipelineBuilder
      ├─> adapter.declare_pipeline(builder, registry)
      │   └─> pass.declare_pipeline(builder, registry)
      │       ├─> builder.vertex_shader(handle)
      │       ├─> builder.fragment_shader(handle)
      │       └─> builder.depth_test(true)
      └─> Returns PipelineBuilder
```

### What's Still Needed

```
Graph Compilation (NEXT STEP)
  └─> graph.compile()
      ├─> For each pass in execution order:
      │   ├─> Get pipeline description
      │   ├─> Get shaders from registry
      │   ├─> Backend: compile shader modules
      │   └─> Backend: create pipeline
      │
      └─> Return CompiledGraph {
              execution_order,
              producers,
              barriers,
              pipelines  // NEW: PassId → PipelineHandle
          }

Execution
  └─> backend.execute_graph(graph, compiled)
      └─> For each pass:
          ├─> Bind pipeline from compiled.pipelines
          └─> Execute pass callback
```

## Testing

✅ All 124 tests pass  
✅ Code passes clippy with no warnings  
✅ Code properly formatted  

## Commits

1. **feat: Integrate ShaderRegistry with RenderGraph** (5565b86)
   - ShaderRegistry integration
   - Shader registration infrastructure

2. **refactor: Use embedded SPIR-V shaders in registry** (72dc1a3)
   - Changed to embedded shaders
   - Simplified initial implementation

3. **feat: Add pipeline description collection to RenderGraph** (f62f03d)
   - PassCallback::declare_pipeline()
   - RenderGraph::get_pipeline_description()
   - PipelineBuilder accessors

## Progress Summary

**Phase 4: Migration - 70% Complete**

✅ Completed:
- ForwardDeclarativePass implementation
- Declarative API migration
- Shader registration
- Pipeline description collection

⏳ Remaining:
- Backend shader compilation
- Pipeline creation from descriptions
- Execution integration
- Testing
- Deprecation of old system

## Next Steps

### Immediate (Next Session)

1. **Backend Shader Module Creation**
   - Add `create_shader_module()` to GraphicsBackend trait
   - Implement in VulkanBackend
   - Handle embedded SPIR-V (for now)

2. **Simple Pipeline Compilation**
   - Collect pipeline descriptions during graph.compile()
   - Create backend pipelines
   - Store in CompiledGraph

3. **Update Execution**
   - Modify execute_graph() to use per-pass pipelines
   - Test with gltf_viewer example

### Future

1. **Runtime HLSL Compilation** (Development Mode)
   - Integrate DXC for Vulkan
   - Support ShaderSource::File
   - Hot-reload capability

2. **DirectX Backend Support**
   - Implement create_shader_module() for DX12
   - HLSL → DXIL compilation

3. **Full Declarative Resources**
   - Vertex layouts from declarations
   - Descriptor layouts from declarations
   - Resource allocation from declarations

## Benefits Achieved

1. **Centralized Shader Management**: All shaders in one registry
2. **Type-Safe Shader References**: ShaderHandle prevents errors
3. **Pipeline Description API**: Passes declare requirements
4. **Foundation for Compilation**: Infrastructure ready for backend integration
5. **Testable Components**: ShaderRegistry and PipelineBuilder tested independently

## Files Changed (Total)

- `src/render_graph/graph.rs` - ShaderRegistry integration, pipeline description
- `src/render_graph/pass.rs` - PassCallback::declare_pipeline()
- `src/render_graph/pipeline.rs` - PipelineBuilder accessors
- `src/pipelines/forward.rs` - Shader registration
- `src/passes/forward_declarative.rs` - Formatting
- `DECLARATIVE_PIPELINE_DESIGN.md` - Design documentation
- `SESSION_SHADER_REGISTRY_2025-10-29.md` - Session notes
- `RENDERGRAPH_REFACTOR_PLAN.md` - Updated status

## Lines of Code

- **Added**: ~450 lines (code + documentation)
- **Modified**: ~50 lines
- **Documentation**: ~300 lines

## Key Insights

1. **Incremental Approach Works**: Using embedded shaders avoids DXC complexity for now
2. **PassCallback Bridge**: Adding declare_pipeline() to PassCallback maintains compatibility
3. **Separation of Concerns**: Pipeline description separate from compilation
4. **Type Safety**: ShaderHandle prevents invalid shader references

## Challenges Overcome

1. **Naming Conflicts**: Resolved vertex_layout() vs get_vertex_layout()
2. **Import Organization**: Added PassKind and PipelineBuilder to graph.rs
3. **Design Complexity**: Simplified by phasing runtime compilation

---

*Session Date: 2025-10-29*
*Final Commit: f62f03d*
*Status: Phase 4 - 70% Complete*

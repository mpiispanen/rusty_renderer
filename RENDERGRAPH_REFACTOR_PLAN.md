# Rendergraph Refactoring Plan

**Status:** 🚧 Phase 4 - Migration in Progress  
**Last Updated:** 2025-10-29  
**Completed Phases:** 1, 2, 3 ✅  
**Current Phase:** 4 - Migrating to declarative API

## Progress Summary

- ✅ Phase 1: Resource descriptors and registry complete
- ✅ Phase 2: DeclarativePass trait and PassBuilder complete
- ✅ Phase 3: ShaderRegistry and PipelineBuilder complete
- 🚧 Phase 4: ForwardDeclarativePass implemented and integrated
- ⏳ Phase 5: Automatic execution planned

## Current State

**Problems:**
- Hardcoded resource paths in passes and pipelines
- Manual resource management scattered across codebase
- Pipelines directly create and manage their own resources
- No central resource tracking or lifetime management
- Difficult to add new passes or change resource flow

**Current Architecture:**
```
Pipeline (forward.rs) 
  ├── Creates textures directly
  ├── Creates buffers directly
  ├── Manages descriptors manually
  └── Hardcoded resource names/paths

RenderPass (passes/forward.rs)
  ├── References pipeline resources
  ├── Manual barrier insertion
  └── No resource dependency tracking
```

## Target State

**Goals:**
- Rendergraph owns ALL resource allocation and lifetime management
- Passes declare their resource requirements declaratively
- Automatic dependency tracking and barrier insertion
- No hardcoded paths - all resources referenced by logical names
- Easy to add new passes and reconfigure the graph

**Target Architecture:**
```
RenderGraph
  ├── Resource Registry (manages all resources)
  │   ├── Images (render targets, depth, textures)
  │   ├── Buffers (uniform, vertex, index)
  │   ├── Samplers
  │   └── Shaders (vertex, fragment, compute)
  │
  ├── Pass Registry
  │   └── Each pass declares:
  │       ├── Input resources (read)
  │       ├── Output resources (write)
  │       ├── Shader requirements (by name)
  │       ├── Pipeline state
  │       └── Execution callback
  │
  └── Compilation & Execution
      ├── Topological sort (dependency order)
      ├── Resource lifetime analysis
      ├── Shader compilation/loading
      ├── Automatic barrier insertion
      └── Resource aliasing/reuse
```

## Migration Strategy

### Phase 1: Resource Descriptors ✅ COMPLETE
- [x] Define resource descriptor types (already exists)
- [x] Add resource creation API to RenderGraph
- [x] Implement resource registry/storage
- [x] Add resource lookup by name/ID

### Phase 2: Pass API Refactoring ✅ COMPLETE
- [x] Update RenderPass to declare resources declaratively (DeclarativePass trait)
- [x] Add methods: `declare_read()`, `declare_write()` (PassBuilder)
- [x] Update PassExecutionContext to provide resources
- [x] DeclarativePassAdapter to bridge new and old APIs

### Phase 3: Pipeline & Shader Integration ✅ COMPLETE
- [x] Shader registry in RenderGraph (ShaderRegistry)
- [x] Passes declare shaders by name (via declare_pipeline)
- [x] Shader compilation infrastructure (ShaderDescriptor, ShaderSource)
- [x] Pipeline state declared in pass (PipelineBuilder)
- [x] Declarative pipeline configuration

### Phase 4: Migration 🚧 IN PROGRESS
- [x] Implement ForwardDeclarativePass
- [x] Migrate ForwardPipeline to use declarative API
- [ ] Register shaders in ShaderRegistry during app initialization
- [ ] Update backends to compile shaders from registry
- [ ] Test rendering with new declarative system
- [ ] Deprecate/remove old ForwardPass

### Phase 5: Execution (PLANNED)
- [ ] Implement automatic barrier insertion
- [ ] Resource lifetime tracking
- [ ] Dependency-ordered execution
- [ ] Resource aliasing for memory efficiency

## Example: Forward Pass Refactored

### Before (Current):
```rust
// Pipeline creates resources
impl ForwardPipeline {
    fn new(backend: &Backend) -> Self {
        let depth_texture = backend.create_texture(...); // Hardcoded
        let uniform_buffer = backend.create_buffer(...);  // Manual
        // ...
    }
}

// Pass manually manages resources
impl ForwardPass {
    fn execute(&self, ctx: &mut ExecutionContext) {
        // Manual barriers
        ctx.insert_barrier(...);
        
        // Direct resource access
        ctx.bind_texture(self.pipeline.depth_texture);
    }
}
```

### After (Target):
```rust
// Pass declares resource requirements
impl RenderPass for ForwardPass {
    fn declare_resources(&self, graph: &mut RenderGraph) {
        // Declare depth buffer
        graph.declare_image("depth", ImageDescriptor {
            extent: Extent::Swapchain,  // Match swapchain size
            format: Format::Depth32Float,
            usage: ImageUsage::DEPTH_ATTACHMENT | ImageUsage::SAMPLED,
        });
        
        // Declare color output
        graph.declare_image("color_output", ImageDescriptor {
            extent: Extent::Swapchain,
            format: Format::Rgba8Unorm,
            usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED,
        });
        
        // Declare uniform buffer
        graph.declare_buffer("camera_uniform", BufferDescriptor {
            size: std::mem::size_of::<CameraUniforms>(),
            usage: BufferUsage::UNIFORM,
        });
    }
    
    fn declare_dependencies(&self, graph: &mut RenderGraph) {
        // Read from scene buffers
        graph.read_buffer("vertex_buffer");
        graph.read_buffer("index_buffer");
        graph.read_texture("albedo_texture");
        
        // Write to outputs
        graph.write_attachment("color_output");
        graph.write_attachment("depth");
    }
    
    fn execute(&self, ctx: &PassExecutionContext) {
        // Resources provided by context
        let depth = ctx.get_image("depth");
        let color = ctx.get_image("color_output");
        let camera_uniforms = ctx.get_buffer("camera_uniform");
        
        // No manual barriers needed - graph handles it!
        ctx.begin_render_pass(/* ... */);
        // ...
    }
}
```

## Implementation Steps

### Step 1: Basic Resource Registry (Week 1)
```rust
// Add to RenderGraph
pub struct RenderGraph {
    resources: HashMap<String, (ResourceId, ResourceDescriptor)>,
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderGraph {
    pub fn declare_image(&mut self, name: &str, desc: ImageDescriptor) -> ResourceId;
    pub fn declare_buffer(&mut self, name: &str, desc: BufferDescriptor) -> ResourceId;
    pub fn get_resource(&self, name: &str) -> Option<ResourceId>;
}
```

### Step 2: Update Pass Trait
```rust
pub trait RenderPass {
    /// Declare resources this pass needs
    fn declare_resources(&self, graph: &mut RenderGraph);
    
    /// Declare resource dependencies
    fn declare_dependencies(&self, pass: &mut PassBuilder);
    
    /// Execute the pass
    fn execute(&self, ctx: &PassExecutionContext);
}
```

### Step 3: PassBuilder API
```rust
pub struct PassBuilder<'a> {
    graph: &'a mut RenderGraph,
    pass_id: PassId,
}

impl<'a> PassBuilder<'a> {
    pub fn read_buffer(&mut self, name: &str);
    pub fn write_buffer(&mut self, name: &str);
    pub fn read_texture(&mut self, name: &str);
    pub fn write_attachment(&mut self, name: &str, layout: ImageLayout);
}
```

### Step 4: ExecutionContext Provides Resources
```rust
pub struct PassExecutionContext<'a> {
    backend: &'a dyn RenderBackend,
    resources: &'a ResourceStorage,
}

impl<'a> PassExecutionContext<'a> {
    pub fn get_image(&self, name: &str) -> &Image;
    pub fn get_buffer(&self, name: &str) -> &Buffer;
    pub fn map_buffer<T>(&mut self, name: &str) -> &mut [T];
}
```

## Benefits After Refactoring

1. **No Hardcoded Paths**
   - Resources referenced by logical names
   - Easy to reconfigure graph

2. **Automatic Resource Management**
   - Graph handles allocation/deallocation
   - Lifetime tracking prevents use-after-free

3. **Automatic Barriers**
   - Graph inserts barriers based on declared dependencies
   - No manual synchronization

4. **Easy to Extend**
   - Adding a pass: declare resources + dependencies
   - No need to manually wire up resources

5. **Memory Efficiency**
   - Resource aliasing where lifetimes don't overlap
   - Automatic memory pooling

6. **Debugging**
   - Graph visualization (what depends on what)
   - Resource tracking (who allocated, who's using)

## Timeline

- **Week 1**: Resource registry + basic API
- **Week 2**: Update one pass (forward) to new API
- **Week 3**: Migrate remaining passes
- **Week 4**: Automatic barrier insertion
- **Week 5**: Resource aliasing + optimization

## Risks & Mitigation

**Risk**: Breaking existing functionality
- **Mitigation**: Incremental migration, keep old code running alongside

**Risk**: Performance regression
- **Mitigation**: Profile before/after, optimize resource lookup

**Risk**: Complex API
- **Mitigation**: Good documentation + examples

---

*Created: 2025-10-29*
*Status: Planning Phase*

## Shader Management

### Current Problems

**Hardcoded in Backend/Pipeline:**
```rust
// Current: Shaders hardcoded in pipeline code
impl ForwardPipeline {
    fn new(backend: &Backend) -> Self {
        let vs_bytes = include_bytes!("../../shaders/compiled/forward.vert.spv");
        let fs_bytes = include_bytes!("../../shaders/compiled/forward.frag.spv");
        
        let vertex_shader = backend.create_shader(vs_bytes);
        let fragment_shader = backend.create_shader(fs_bytes);
        // ...
    }
}
```

**Issues:**
- Shader paths hardcoded in multiple places
- No central shader management
- Difficult to swap shaders at runtime
- Can't reuse shaders across passes
- Hard to hot-reload shaders for development

### Target: Declarative Shader API

**Shader Registry:**
```rust
// Shaders registered with graph by name
graph.register_shader("forward.vert", ShaderDescriptor {
    source: ShaderSource::File("shaders/hlsl/forward.hlsl"),
    entry_point: "vs_main",
    stage: ShaderStage::Vertex,
    backend_compile: true,  // Let backend compile (HLSL -> SPIR-V/DXIL)
});

graph.register_shader("forward.frag", ShaderDescriptor {
    source: ShaderSource::File("shaders/hlsl/forward.hlsl"),
    entry_point: "ps_main",
    stage: ShaderStage::Fragment,
    backend_compile: true,
});

// Or use pre-compiled
graph.register_shader("forward.vert", ShaderDescriptor {
    source: ShaderSource::Compiled("shaders/compiled/forward.vert.spv"),
    stage: ShaderStage::Vertex,
    backend_compile: false,
});
```

**Pass Declares Shaders:**
```rust
impl RenderPass for ForwardPass {
    fn declare_pipeline(&self, builder: &mut PipelineBuilder) {
        builder
            .vertex_shader("forward.vert")
            .fragment_shader("forward.frag")
            .vertex_layout(VertexLayout::Pos3Color4Uv2)
            .depth_test(true)
            .depth_write(true)
            .cull_mode(CullMode::Back);
    }
}
```

### Shader Source Types

```rust
pub enum ShaderSource {
    /// Load from file (HLSL, GLSL, etc.)
    File(&'static str),
    
    /// Pre-compiled bytecode (SPIR-V, DXIL)
    Compiled(&'static str),
    
    /// Inline source code
    Inline(&'static str),
    
    /// Embedded at compile-time
    Embedded(&'static [u8]),
}

pub struct ShaderDescriptor {
    pub source: ShaderSource,
    pub entry_point: &'static str,
    pub stage: ShaderStage,
    pub backend_compile: bool,  // True = compile at runtime, False = use as-is
}
```

### Shader Compilation Strategy

**Development Mode:**
- Load HLSL source from disk
- Compile at runtime (hot-reload support)
- Cache compiled shaders

**Release Mode:**
- Use pre-compiled SPIR-V/DXIL
- Embedded in binary (`include_bytes!`)
- No runtime compilation overhead

### Implementation Plan

**Step 1: Shader Registry**
```rust
pub struct ShaderRegistry {
    shaders: HashMap<String, ShaderHandle>,
    descriptors: HashMap<String, ShaderDescriptor>,
}

impl ShaderRegistry {
    pub fn register(&mut self, name: &str, desc: ShaderDescriptor);
    pub fn get(&self, name: &str) -> Option<ShaderHandle>;
    pub fn compile(&mut self, backend: &dyn Backend);
}
```

**Step 2: Pipeline Builder**
```rust
pub struct PipelineBuilder<'a> {
    shaders: Vec<String>,  // Shader names
    vertex_layout: Option<VertexLayout>,
    depth_state: DepthState,
    // ... other pipeline state
}

impl<'a> PipelineBuilder<'a> {
    pub fn vertex_shader(&mut self, name: &str) -> &mut Self;
    pub fn fragment_shader(&mut self, name: &str) -> &mut Self;
    pub fn compute_shader(&mut self, name: &str) -> &mut Self;
}
```

**Step 3: Pass Integration**
```rust
pub trait RenderPass {
    fn declare_pipeline(&self, builder: &mut PipelineBuilder) {
        // Default: no shaders (compute/transfer pass)
    }
}
```

### Example: Forward Pass Refactored

**Before:**
```rust
// Hardcoded in pipeline
let vs = include_bytes!("../../shaders/compiled/forward.vert.spv");
let fs = include_bytes!("../../shaders/compiled/forward.frag.spv");
```

**After:**
```rust
// Application setup
graph.register_shader("forward.vert", ShaderDescriptor {
    source: ShaderSource::File("shaders/hlsl/forward.hlsl"),
    entry_point: "vs_main",
    stage: ShaderStage::Vertex,
    backend_compile: true,
});

graph.register_shader("forward.frag", ShaderDescriptor {
    source: ShaderSource::File("shaders/hlsl/forward.hlsl"),
    entry_point: "ps_main",
    stage: ShaderStage::Fragment,
    backend_compile: true,
});

// Pass declares what it needs
impl RenderPass for ForwardPass {
    fn declare_pipeline(&self, builder: &mut PipelineBuilder) {
        builder
            .vertex_shader("forward.vert")
            .fragment_shader("forward.frag");
    }
}
```

### Benefits

1. **No Hardcoded Paths**
   - Shaders referenced by name
   - Easy to swap implementations

2. **Shader Reuse**
   - Same shader used across multiple passes
   - Compiled once, used many times

3. **Hot Reload**
   - Watch shader files in dev mode
   - Recompile on change
   - No app restart needed

4. **Backend Agnostic**
   - Graph manages shader source
   - Backend compiles to native format
   - Same API for HLSL, GLSL, SPIR-V

5. **Development vs Release**
   - Dev: Source files, runtime compilation
   - Release: Pre-compiled, embedded

### Shader Variants

For future advanced usage:
```rust
// Different shader variants for different quality levels
graph.register_shader_variant("forward.frag", "low_quality", ...);
graph.register_shader_variant("forward.frag", "high_quality", ...);

// Pass can select variant
builder.fragment_shader("forward.frag")
       .variant(quality_setting);
```


## GitHub Issues

### Phase 1: Resource Registry
- #75 - Implement resource registry and name-based lookup
- #76 - Extend resource descriptors with all required fields

### Phase 2: Declarative Pass API  
- #77 - Add declarative methods to RenderPass trait
- #78 - Implement PassBuilder for dependency declaration
- #79 - Update PassExecutionContext to provide resources

### Phase 3: Shader & Pipeline Integration
- #84 - Implement shader registry and ShaderDescriptor
- #80 - Add PipelineBuilder for declarative pipeline configuration
- #81 - Extend RenderPass trait with pipeline declaration

### Phase 4: Migration
- #85 - Migrate ForwardPass to declarative API

### Phase 5: Automatic Execution
- #82 - Implement pass dependency analysis and topological sort
- #83 - Implement automatic pipeline barrier insertion

### Related Issues
- #65 - Design and implement pass requirement system (existing)
- #66 - Implement automatic resource allocation in render graph (existing)

---

*Issues created: 2025-10-29*
*Ready to implement!*

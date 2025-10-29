# Rendergraph Refactoring Plan

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
  │   └── Samplers
  │
  ├── Pass Registry
  │   └── Each pass declares:
  │       ├── Input resources (read)
  │       ├── Output resources (write)
  │       ├── Pipeline requirements
  │       └── Execution callback
  │
  └── Compilation & Execution
      ├── Topological sort (dependency order)
      ├── Resource lifetime analysis
      ├── Automatic barrier insertion
      └── Resource aliasing/reuse
```

## Migration Strategy

### Phase 1: Resource Descriptors
- [x] Define resource descriptor types (already exists)
- [ ] Add resource creation API to RenderGraph
- [ ] Implement resource registry/storage
- [ ] Add resource lookup by name/ID

### Phase 2: Pass API Refactoring
- [ ] Update RenderPass to declare resources declaratively
- [ ] Add methods: `declare_read()`, `declare_write()`, `declare_attachment()`
- [ ] Remove manual resource management from passes
- [ ] Update PassExecutionContext to provide resources

### Phase 3: Pipeline Integration
- [ ] Move pipeline resource creation to RenderGraph
- [ ] Pipelines declare resource requirements, don't create them
- [ ] Update descriptor set management
- [ ] Remove hardcoded resource paths from pipelines

### Phase 4: Scene/Material Resources
- [ ] Scene loader registers resources with graph
- [ ] Materials reference resources by ID, not direct handles
- [ ] Texture/buffer upload through graph API

### Phase 5: Execution
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

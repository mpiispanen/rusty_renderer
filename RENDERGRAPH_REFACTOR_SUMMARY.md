# Rendergraph Refactoring - Quick Reference

## Vision: Declarative, Data-Driven Rendering

Everything managed by the **RenderGraph** - no hardcoded paths anywhere.

## What Gets Managed

1. **Resources** (Textures, Buffers, Samplers)
2. **Shaders** (Vertex, Fragment, Compute)
3. **Dependencies** (Automatic ordering)
4. **Synchronization** (Automatic barriers)
5. **Lifetimes** (Allocation, aliasing, deallocation)

## Example: Forward Pass

### Before (Current) ❌
```rust
// Hardcoded everywhere
let depth = backend.create_texture("depth.png");  // Hardcoded path
let vs = include_bytes!("shaders/forward.vert.spv");  // Hardcoded
let fs = include_bytes!("shaders/forward.frag.spv");  // Hardcoded

// Manual synchronization
ctx.barrier(...);  // Manual
```

### After (Target) ✅
```rust
// Setup: Register resources and shaders once
graph.register_shader("forward.vert", ShaderDescriptor {
    source: ShaderSource::File("shaders/hlsl/forward.hlsl"),
    entry_point: "vs_main",
    stage: ShaderStage::Vertex,
});

// Pass: Declare what you need
impl RenderPass for ForwardPass {
    fn declare_resources(&self, graph: &mut RenderGraph) {
        graph.declare_image("depth", ImageDesc { ... });
        graph.declare_image("color", ImageDesc { ... });
    }
    
    fn declare_dependencies(&self, pass: &mut PassBuilder) {
        pass.read_texture("albedo");
        pass.write_attachment("color");
        pass.write_attachment("depth");
    }
    
    fn declare_pipeline(&self, pipeline: &mut PipelineBuilder) {
        pipeline.vertex_shader("forward.vert")
                .fragment_shader("forward.frag")
                .depth_test(true);
    }
    
    fn execute(&self, ctx: &PassExecutionContext) {
        let depth = ctx.get_image("depth");
        let color = ctx.get_image("color");
        // No barriers needed - graph handles it!
    }
}
```

## Key Principles

### 1. Declare, Don't Create
Passes **declare** what they need, RenderGraph **creates** it.

### 2. Reference by Name
No direct handles, all resources referenced by logical names.

### 3. Automatic Everything
- Dependencies → Automatic ordering
- Resource usage → Automatic barriers
- Overlapping lifetimes → Automatic aliasing

### 4. Backend Agnostic
Graph manages high-level concepts, backend handles low-level details.

## Benefits

| Before | After |
|--------|-------|
| Hardcoded paths everywhere | Logical names |
| Manual resource creation | Declarative requirements |
| Manual barriers | Automatic synchronization |
| Difficult to change | Easy to reconfigure |
| Copy-paste to add pass | Declare requirements |
| Backend-specific code | Backend-agnostic API |

## Migration Strategy

**Incremental, Safe Approach:**
1. Add new API alongside old code
2. Migrate one pass at a time
3. Keep old code working until complete
4. Remove old code when fully migrated

**Phase by Phase:**
- Phase 1: Resource registry
- Phase 2: Pass declaration API
- Phase 3: Shader registry
- Phase 4: Scene integration
- Phase 5: Automatic execution

## Development vs Release

### Development Mode
- Load shaders from disk
- Runtime compilation
- Hot-reload support
- Validation layers

### Release Mode
- Pre-compiled shaders embedded
- No runtime compilation
- Optimized barriers
- Minimal overhead

## Future Possibilities

Once the foundation is in place:
- **Graph Visualization**: Render dependency graph
- **Shader Variants**: Low/Medium/High quality
- **Multi-frame Resources**: Ping-pong buffers
- **Resource Pooling**: Automatic memory management
- **Async Compilation**: Background shader compilation
- **GPU-driven Rendering**: Compute-based culling

---

**See RENDERGRAPH_REFACTOR_PLAN.md for full details**

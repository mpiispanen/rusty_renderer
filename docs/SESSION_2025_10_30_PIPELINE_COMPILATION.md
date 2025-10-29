# Pipeline Compilation Session - 2025-10-30

## Goal

Implement backend pipeline compilation from declarative pipeline descriptions.

## Current State

We have:
- ✅ ShaderRegistry in RenderGraph
- ✅ Shaders registered during graph build
- ✅ ForwardDeclarativePass with `declare_pipeline()` 
- ✅ PipelineBuilder collecting pipeline state
- ✅ `get_pipeline_description()` to retrieve requirements
- ❌ No pipeline compilation in CompiledGraph
- ❌ Backends don't create pipelines from descriptions
- ❌ Execution uses hardcoded pipelines

## Implementation Plan

### Step 1: Pipeline Storage in CompiledGraph

Add pipeline handles to CompiledGraph to store compiled pipelines per pass.

**Changes to `src/render_graph/graph.rs`:**

```rust
// Add pipeline handle type
pub type PipelineHandle = usize;

// Update CompiledGraph
pub struct CompiledGraph {
    pub execution_order: Vec<PassId>,
    pub producers: HashMap<ResourceId, PassId>,
    pub barriers: Vec<Barrier>,
    // NEW: Store compiled pipelines
    pub pipelines: HashMap<PassId, PipelineHandle>,
}
```

### Step 2: Collect Pipeline Descriptions During Compilation

Update `compile()` to collect pipeline descriptions.

**Changes to `RenderGraph::compile()`:**

```rust
pub fn compile(&mut self) -> Result<CompiledGraph> {
    // ... existing code ...
    
    // Collect pipeline descriptions
    let mut pipeline_descriptions = HashMap::new();
    for &pass_id in &execution_order {
        if let Some(desc) = self.get_pipeline_description(pass_id) {
            pipeline_descriptions.insert(pass_id, desc);
        }
    }
    
    Ok(CompiledGraph {
        execution_order,
        producers,
        barriers,
        pipeline_descriptions,  // Store for now, compile in backend later
    })
}
```

### Step 3: Backend Pipeline Compilation (Deferred)

For now, backend pipeline compilation is deferred. We'll store the descriptions
and compile them lazily when needed. This allows us to:

1. Test the pipeline description collection
2. Verify the descriptions are correct
3. Implement backend compilation incrementally

## Testing Strategy

1. Run existing tests to ensure no regressions
2. Add test to verify pipeline descriptions are collected
3. Log pipeline descriptions during graph compilation
4. Verify descriptions match expected pass requirements

## Next Session Tasks

After this session:
1. Implement backend pipeline creation from descriptions
2. Update execute_graph to use compiled pipelines
3. Remove hardcoded pipeline creation from backends
4. Test end-to-end rendering

## Timeline

- This session: Pipeline description collection (~1 hour)
- Next session: Backend pipeline creation (~2-3 hours)
- Following: Integration and testing (~2 hours)


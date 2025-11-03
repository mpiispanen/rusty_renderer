# Vulkan/DirectX Debug Session - Part 8
## Date: 2025-11-02

## Investigation: DirectX Rendering Black Screen

### Current Status
- **Vulkan**: Rendering correctly (shows colored cube)
- **DirectX**: Renders only clear color (transparent black in frame capture)

### Root Cause Identified

DirectX is NOT using the render graph's compiled pipelines! The issue is in `execute_graph`:

```rust
// Line 1596-1599 in src/backends/directx/dx12_impl.rs
let pipeline_state = self
    .pipeline_state
    .as_ref()
    .context("Pipeline state not initialized")?;
```

This uses `self.pipeline_state` which is the OLD embedded HLSL pipeline, not the render graph pipelines.

### What Vulkan Does Right

Vulkan properly compiles pipelines from the render graph's pipeline descriptions:

```rust
// src/backends/vulkan/mod.rs:3111-3120
for (pass_id, builder) in &compiled.pipeline_descriptions {
    if !self.pipeline_cache.contains_key(pass_id) {
        log::debug!("Compiling pipeline for pass {:?}", pass_id);
        let pipeline =
            self.compile_pipeline_from_builder(builder, graph.shader_registry(), *pass_id)?;
        self.pipeline_cache.insert(*pass_id, pipeline);
    } else {
        log::debug!("Using cached pipeline for pass {:?}", pass_id);
    }
}
```

### What DirectX Needs

DirectX needs to:

1. **Implement `compile_pipeline_from_builder`** - Convert `GraphicsPipelineBuilder` to D3D12 pipeline state
2. **Add `pipeline_cache`** - Store compiled pipelines by PassId  
3. **Update `execute_graph`** - Use cached pipelines instead of `self.pipeline_state`
4. **Load DXIL shaders** - Read pre-compiled .dxil files from the render graph's shader registry

### Implementation Steps

1. Add pipeline cache to DirectXBackendImpl:
   ```rust
   pipeline_cache: HashMap<PassId, ID3D12PipelineState>,
   ```

2. Implement `compile_pipeline_from_builder`:
   - Parse shader paths from builder
   - Load .dxil bytecode
   - Create root signature from bind group layouts
   - Create D3D12_GRAPHICS_PIPELINE_STATE_DESC
   - Call CreateGraphicsPipelineState

3. In execute_graph:
   - Before rendering, compile/cache all pipelines from `compiled.pipeline_descriptions`
   - For each pass, look up the correct pipeline from cache
   - SetPipelineState with the pass-specific pipeline

### Evidence

**DirectX log shows draw call happening:**
```
DirectX Draw: 36 vertices, 1 instances
```

**But frame capture shows transparent black** - because the pipeline doesn't render anything (wrong shaders).

**Vulkan works** - because it loads the correct SPIR-V shaders from the compiled HLSL.

### Next Steps

1. Implement DirectX pipeline compilation from render graph
2. Add DXIL shader loading  
3. Update execute_graph to use per-pass pipelines
4. Test that both backends render identically

### Notes

- The unified HLSL shader compilation is working correctly
- Build produces both .spv (for Vulkan) and .dxil (for DirectX) files
- Vulkan successfully loads .spv files
- DirectX just needs to load .dxil files the same way

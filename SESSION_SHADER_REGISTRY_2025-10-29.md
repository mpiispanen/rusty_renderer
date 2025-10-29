# Shader Registry Integration Session - 2025-10-29

## Summary

Integrated the ShaderRegistry with the RenderGraph system, enabling centralized shader management and declarative shader usage in passes.

## Changes Made

### 1. RenderGraph Integration

**File: `src/render_graph/graph.rs`**

- Added `ShaderRegistry` field to `RenderGraph` struct
- Added `register_shader()` method for registering shaders by name
- Added `shader_registry()` and `shader_registry_mut()` accessors
- Import shader types in graph module

### 2. Shader Registration in Pipeline

**File: `src/pipelines/forward.rs`**

- Added shader registration in `build_graph()` method
- Registered `forward.vert` shader (VSMain entry point)
- Registered `forward.frag` shader (PSMain entry point)
- Both shaders use `shaders/hlsl/forward.hlsl` as source
- Backend compilation enabled for runtime compilation

### 3. ForwardDeclarativePass Updates

**File: `src/passes/forward_declarative.rs`**

- Already had code to use shaders from registry
- Uses `registry.get_handle("forward.vert")` to get shader handles
- Adds shaders to PipelineBuilder via `vertex_shader()` and `fragment_shader()`
- Provides warning logs if shaders not found in registry

## Architecture

### Shader Registration Flow

```
ForwardPipeline::build_graph()
  └─> RenderGraph::new()
      └─> ShaderRegistry::new()
  
  └─> graph.register_shader("forward.vert", ...)
      └─> ShaderRegistry::register()
          └─> Returns ShaderHandle
  
  └─> graph.register_shader("forward.frag", ...)
      └─> ShaderRegistry::register()
          └─> Returns ShaderHandle
```

### Shader Usage Flow (Current)

```
ForwardDeclarativePass::declare_pipeline(builder, registry)
  └─> registry.get_handle("forward.vert")
      └─> Returns Result<ShaderHandle>
  
  └─> builder.vertex_shader(handle)
  
  └─> registry.get_handle("forward.frag")
      └─> Returns Result<ShaderHandle>
  
  └─> builder.fragment_shader(handle)
```

## Testing

- ✅ All 124 tests pass
- ✅ Code passes clippy with no warnings
- ✅ Code properly formatted

## What's Working

1. **Shader Registry**: Centralized shader storage and lookup by name
2. **Shader Registration**: Shaders registered during graph build
3. **Declarative Shader Usage**: Passes can reference shaders by name
4. **Type Safety**: ShaderHandle provides type-safe shader references

## What's Not Yet Implemented

### Backend Compilation

The backends don't yet compile shaders from the registry. Currently:

- Backends still use hardcoded shader includes
- Pipeline creation doesn't use registry-based shaders
- Shader compilation isn't integrated with the declarative system

**What's Needed:**

1. **Pipeline Compilation Hook**: When compiling the render graph, collect pipeline declarations
2. **Shader Compilation**: Backends should:
   - Retrieve shader descriptors from registry
   - Load shader source (File/Compiled/Embedded)
   - Compile to backend format (SPIR-V for Vulkan, DXIL for DirectX)
   - Cache compiled shaders in registry
3. **Pipeline Creation**: Create backend pipelines using compiled shaders from registry

### Execution Integration

The execution system doesn't yet use declarative pipelines:

- `execute_graph()` still binds hardcoded pipelines
- No mapping from passes to their pipelines
- Resource bindings still manual

**What's Needed:**

1. **Pipeline Storage**: Map PassId → Pipeline
2. **Execution Updates**: Bind correct pipeline per pass
3. **Resource Binding**: Automatic based on declarations

## Next Steps

### Immediate (Phase 4 Completion)

1. **Backend Shader Compilation**
   - Add `compile_shader()` method to GraphicsBackend trait
   - Implement in Vulkan backend (DXC for HLSL → SPIR-V)
   - Implement in DirectX backend (DXC for HLSL → DXIL)

2. **Pipeline Creation from Declarations**
   - Add pipeline creation to graph compilation
   - Store pipelines in CompiledGraph
   - Map PassId to Pipeline handle

3. **Update Execution**
   - Modify `execute_graph()` to use per-pass pipelines
   - Test rendering with declarative system

### Future (Phase 5)

1. **Automatic Resource Management**
   - Resource allocation from declarations
   - Automatic barrier insertion
   - Resource aliasing optimization

2. **Hot Reloading** (Development Mode)
   - Watch shader files for changes
   - Recompile and update pipelines
   - No app restart needed

## Benefits Achieved

1. **Centralized Management**: All shaders registered in one place
2. **No Hardcoded Paths in Passes**: Passes reference shaders by name
3. **Type Safety**: ShaderHandle prevents invalid shader references
4. **Testability**: ShaderRegistry tested independently
5. **Foundation for Hot Reload**: Infrastructure ready for file watching

## Files Changed

- `src/render_graph/graph.rs` - Added ShaderRegistry integration
- `src/pipelines/forward.rs` - Register shaders during build
- `src/passes/forward_declarative.rs` - Formatting cleanup

## Commit

```
feat: Integrate ShaderRegistry with RenderGraph

- Add ShaderRegistry field to RenderGraph
- Add register_shader() and shader_registry() methods to RenderGraph
- Register forward shaders in ForwardPipeline::build_graph
- ForwardDeclarativePass now uses shaders from registry
- All tests pass, code formatted and linted

This completes Phase 4 step: shader registration during app initialization.
Next step: Backend compilation of shaders from registry.
```

## Status

**Phase 4 Progress: 60% Complete**

- ✅ ForwardDeclarativePass implementation
- ✅ Declarative API migration
- ✅ Shader registration
- ⏳ Backend compilation (next)
- ⏳ Testing
- ⏳ Deprecation of old system

---

*Session Date: 2025-10-29*
*Commit: 5565b86*

# Session: Migration to App-based Architecture
**Date:** 2025-11-01
**Focus:** Migrating from old pipeline system to render graph-based App architecture

## Summary

Successfully migrated the main entry point from the old `ApplicationRunner` + `PipelineFactory` system to the modern `App` + `RenderGraph` architecture. This completes a major milestone in our render graph refactoring.

## Changes Made

### 1. Main Entry Point Migration
- **Before**: Used `ApplicationRunner::from_args()` which relied on `ApplicationArgs` and `PipelineFactory`
- **After**: Uses `Config::parse_args()` and `App::run()` which builds render graphs declaratively
- Old system created `ForwardPipeline` objects that managed their own resources
- New system uses `RenderGraph` with declarative passes like `ForwardSimplePass` and `TrianglePass`

### 2. Render Graph Validation Fix
**Problem**: Resources created with `declare_buffer_with_data()` were failing validation with "Resource has no producer" error.

**Root Cause**: Resources with initial data (vertex buffers, uniform buffers) don't have an explicit producer pass - they're uploaded directly. The validation was treating them as regular resources that need a producer.

**Solution**: Modified graph compilation validation to skip producer checks for:
- External resources (`resource.external == true`)
- Resources with initial data (`resource.init_data != ResourceInitData::None`)

These resources act as implicit producers since they provide their own data.

```rust
// Skip validation for external resources or resources with init data
// (they act as implicit producers)
if resource.external || !matches!(resource.init_data, ResourceInitData::None) {
    continue;
}
```

### 3. Architecture Simplification

**Removed Dependencies:**
- No longer uses `application::ApplicationRunner`
- No longer uses `pipelines::PipelineFactory`
- No longer uses old `ForwardPipeline`

**Current Flow:**
```
main.rs
  └─> Config::parse_args()
      └─> App::new(config)
          └─> App::run_headless() or App::run()
              ├─> load_scene()
              ├─> build_render_graph()
              │   ├─> TrianglePass (for debug scenes)
              │   └─> ForwardSimplePass (for mesh scenes)
              └─> loop:
                  ├─> graph.compile()
                  ├─> backend.execute_graph(&graph, &compiled)
                  └─> capture_screenshot()
```

## Testing

Verified both rendering paths work correctly:

### Triangle Scene (Debug)
```bash
cargo run -- --scene triangle --headless --max-frames 1 --screenshot test.png
```
- ✅ Renders successfully
- ✅ Uses TrianglePass
- ✅ Screenshot captured

### Cube Scene (Forward Rendering)
```bash
cargo run -- --scene cube --headless --max-frames 1 --screenshot test.png
```
- ✅ Renders successfully
- ✅ Uses ForwardSimplePass
- ✅ Vertex buffer created with init_data
- ✅ Camera and lighting uniforms created with init_data
- ✅ Screenshot captured
- ✅ All 36 vertices rendered

### Test Suite
- ✅ All 129 tests passing
- ✅ Clippy clean (no warnings)
- ✅ Format check passing

## Architecture Benefits

### Before (Old System)
```rust
// main.rs
ApplicationRunner::from_args()
  └─> PipelineFactory::create("forward")
      └─> ForwardPipeline::new()
          ├─> Manually creates buffers
          ├─> Manually manages descriptors
          ├─> Hardcoded shader paths
          └─> Tight coupling to backend
```

### After (New System)
```rust
// main.rs
App::run(Config)
  └─> build_render_graph()
      └─> ForwardSimplePass::builder()
          .vertex_buffer(declare_buffer_with_data(...))
          .camera_buffer(declare_buffer_with_data(...))
          .build(&mut graph)
              ├─> Declares resources
              ├─> Declares pipeline requirements
              ├─> Registers shaders
              └─> Graph handles allocation
```

**Key Improvements:**
1. **Declarative** - Passes declare what they need, graph manages it
2. **Backend-agnostic** - Passes don't know about Vulkan/DirectX
3. **Automatic resource management** - Graph allocates and tracks lifetime
4. **Automatic pipeline compilation** - Backends compile from PipelineBuilder
5. **Automatic shader compilation** - ShaderRegistry manages compilation
6. **No hardcoded paths** - Everything declared in passes

## What Works Now

### Render Graph Features
- ✅ Declarative resource creation
- ✅ Declarative pipeline creation
- ✅ Automatic resource allocation
- ✅ Resource initialization with data upload
- ✅ Shader registry and compilation
- ✅ Pipeline caching per pass
- ✅ Pass execution with resource lookup
- ✅ Backend resource mapping

### Passes
- ✅ TrianglePass - Simple debug rendering
- ✅ ForwardSimplePass - Forward rendering with lighting

### Resource Types
- ✅ Buffers (vertex, index, uniform)
- ✅ Images (color attachments, depth buffers)
- ✅ Resources with initial data
- ✅ External/imported resources

### Backends
- ✅ Vulkan - Full render graph support
- ⚠️ DirectX - Resource allocation working, pipeline compilation TBD

## Current State

### Working
- Main application uses render graph exclusively
- Triangle scene renders correctly
- Cube scene with lighting renders correctly
- Resource allocation and upload working
- Pipeline compilation from descriptors (Vulkan)
- Pass execution with resource access

### TODO (Next Steps)
1. **DirectX Pipeline Compilation**: Add `compile_pipeline_from_builder` to DirectX backend
2. **Remove Old Code**: Clean up unused `ApplicationRunner`, `PipelineFactory`, old pipeline code
3. **Additional Passes**: Implement more render passes (shadow maps, post-processing, etc.)
4. **Index Buffers**: Test indexed rendering (currently using vertex count)
5. **Texture Support**: Add texture uploads and sampling
6. **Descriptor Management**: Improve descriptor set allocation

## Issues Resolved

- ✅ Issue #85: Phase 4.1: Migrate ForwardPass to declarative API
- ✅ Part of Issue #87: Phase 4.2: Resource allocation and mapping

## Files Modified

- `src/main.rs` - Migrated from ApplicationRunner to App
- `src/render_graph/graph.rs` - Fixed validation for resources with init data

## Commits

1. `085b0ac` - refactor: Migrate main entry point to use App instead of ApplicationRunner

## References

- Previous Session: RESOURCE_UPLOAD_COMPLETE.md
- Architecture Plan: RENDERGRAPH_REFACTOR_PLAN.md
- Next Steps: DirectX pipeline compilation, cleanup old code

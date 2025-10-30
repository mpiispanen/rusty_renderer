# ForwardPass Migration Session - Part 2
## Date: 2025-10-30

## Context

Continuing the migration of ForwardPass to use the declarative render graph API. The user wants to:
1. Remove all hardcoded paths  
2. Define everything declaratively in renderpasses
3. Have rendergraph handle resource management
4. Get rendering working end-to-end with ForwardSimplePass

## Current Architecture

### Passes
- `ForwardPass` - Legacy pass (old)
- `ForwardDeclarativePass` - Intermediate (transitional)
- `ForwardRenderPass` - Current (takes backend Buffer/Texture objects + ResourceIds)
- `ForwardSimplePass` - Target (takes only ResourceIds, fully declarative)

### Resource Management
- **Current:** Pipelines create backend buffers/textures directly
- **Target:** Render graph creates and manages all resources via ResourceIds

## Progress This Session

### ✅ Completed

1. **External Resource Import API**
   - Added `import_buffer()` to RenderGraph
   - Added `import_image()` to RenderGraph
   - Added `external` flag to Resource struct
   - Added `mark_external()` and `is_external()` methods
   
   ```rust
   // Import existing backend resources
   let camera_id = graph.import_buffer("camera", 128, 
       BufferUsageFlags::new(BufferUsageFlags::UNIFORM));
   let albedo_id = graph.import_image("albedo", Format::Rgba8Unorm, ...);
   ```

2. **Vertex Layout Declaration**
   - Added complete vertex layout to ForwardSimplePass
   - Defined all 4 attributes: position, normal, uv, color
   - Matches Vertex struct (48 bytes total)
   - Fixes validation errors for pipeline compilation

3. **Created tracking issues**
   - #86: Extract lights from scenes instead of hardcoding
   - #87: Add render graph resource allocation and mapping

### Findings

1. **Hardcoded lights investigation**
   - ForwardPipeline already gets lights from scene via `LightingUniforms::from_scene()`
   - Cube scene has no `[lighting]` section - that's why we see no lights
   - Not actually hardcoded in pipeline, but scenes may need lighting data

2. **Architecture gap identified**
   - ForwardSimplePass takes ResourceIds (good, declarative)
   - But backends need ResourceId → backend resource mapping
   - Render graph needs resource allocation phase during compilation
   - This is a larger architectural change (#87)

## Next Steps

### Immediate
1. **Test ForwardSimplePass with scenes**
   - Verify rendering works with vertex layout
   - Check for remaining validation errors
   - Compare output with ForwardRenderPass

### Short Term (Issue #87)
1. **Resource Allocation System**
   - Add allocation phase to graph compilation
   - Create ResourceId → backend resource map
   - Update backends to resolve ResourceIds

2. **Update ForwardPipeline**
   - Use import_buffer/import_image for transition
   - Eventually migrate to pure ResourceId-based approach
   - Remove direct backend buffer creation

### Long Term
1. **Full Declarative API**
   - All resources managed by render graph
   - No direct backend resource creation in pipelines
   - Automatic resource lifetime management

## Architecture Notes

### Current ForwardPipeline Flow
```
Pipeline::build_graph():
  1. Create backend buffers (camera, lighting, vertices, materials)
  2. Create ResourceIds for output (color, depth)
  3. Build ForwardRenderPass with buffers + ResourceIds
  4. Backend executes graph, maps ResourceIds to resources

Problems:
  - Mixed backend resources and ResourceIds
  - Pipeline must manage backend resource lifetime
  - Not fully declarative
```

### Target ForwardSimplePass Flow
```
Pipeline::build_graph():
  1. Declare ResourceIds for ALL resources (buffers, textures, outputs)
  2. Build ForwardSimplePass with only ResourceIds
  3. Render graph compiles and allocates resources
  4. Backend executes with resource map

Benefits:
  - Fully declarative
  - Render graph manages all resources
  - Pipeline just describes what's needed
  - Cleaner separation of concerns
```

### Transition Strategy
```
Phase 1 (Current): ✅
  - Add import_buffer/import_image API
  - Add vertex layout declaration
  - Keep ForwardRenderPass for now
  - Pipelines import backend resources as ResourceIds

Phase 2 (Issue #87):
  - Add resource allocation to graph compilation
  - Add ResourceId → backend resource mapping
  - Update backends to resolve ResourceIds

Phase 3 (Issue #85):
  - Migrate ForwardPipeline to ForwardSimplePass
  - Remove ForwardRenderPass
  - Pure ResourceId-based architecture
```

## Code Quality

- ✅ 127 tests passing
- ✅ Zero clippy warnings
- ✅ Properly formatted
- ✅ Clean commit history

## Commits

1. `feat: Add external resource import capability to render graph`
   - Added import_buffer() and import_image()
   - Added external flag to Resource
   - Foundation for resource import workflow

2. `feat: Add vertex layout declaration to ForwardSimplePass`
   - Complete vertex layout with 4 attributes
   - Fixes validation errors
   - Enables pipeline compilation

## References

- Issue #85: Phase 4.1: Migrate ForwardPass to declarative API
- Issue #86: Extract lights from scenes instead of hardcoding
- Issue #87: Phase 4.2: Add render graph resource allocation and mapping
- Previous session: SESSION_FORWARD_MIGRATION_2025-10-30.md

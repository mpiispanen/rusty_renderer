# Architecture Refactor Plan - Data-Driven Rendering

**Date:** 2025-10-27  
**Status:** Planning  
**Goal:** Remove all hardcoded rendering, establish CI rendering comparisons

## Current State

### What Works
- ✅ Vulkan backend: Renders textured cube with lighting correctly
- ✅ DirectX backend: Renders textured cube with lighting correctly  
- ✅ Both backends use render graph for execution
- ✅ GLTF loading working
- ✅ Forward rendering pass implemented
- ✅ Scene system with TOML files

### What's Hardcoded (Problems)
1. **Shaders** - Shaders are compiled from fixed paths, not defined by pipeline templates
2. **Pipeline State** - Depth testing, culling, blending all hardcoded in backend initialization
3. **Resource Bindings** - Descriptor layouts hardcoded in backends
4. **Vertex Format** - Single hardcoded vertex format across all rendering
5. **Render Targets** - Output targets and formats not configurable

### Visual Differences Between Backends
- Slight color differences between Vulkan and DirectX (may be due to coordinate system handling)
- Need automated comparison to detect regressions

## Architecture Goals

### Vision
**All rendering should be data-driven:**
- Scene files define what objects to render
- Pipeline templates define how to render them
- Render graph manages resources automatically
- No hardcoded shaders, bindings, or pipeline state in backends

### Separation of Concerns

```
Scene Files (TOML/GLTF)
  ↓ defines objects, materials, lights
Pipeline Templates (TOML)
  ↓ defines shaders, bindings, state
Render Graph
  ↓ manages resources, dependencies
Backends
  ↓ execute low-level API calls
```

## Phase 1: CI Rendering & Comparison

**Goal:** Establish automated visual testing infrastructure

### Tasks
- [ ] Update CI to render test scenes on both Vulkan and DirectX
- [ ] Enable headless rendering for CI
- [ ] Capture screenshots from both backends
- [ ] Use FLIP comparison for backend parity validation
- [ ] Create golden reference images
- [ ] Fail CI if backends diverge or regress from golden images

### Deliverables
- CI workflow that renders test scenes
- Automated visual regression detection
- Backend parity validation
- Golden image library

### Acceptance Criteria
- CI runs successfully on every commit
- Visual differences detected automatically
- Both backends produce identical or near-identical output
- Regression reports generated with diff images

## Phase 2: Remove Hardcoded Shaders

**Goal:** Shaders defined by pipeline templates, not hardcoded in backends

### Current State
- Shaders compiled from `shaders/` directory during build
- Shader paths hardcoded in backend initialization
- No runtime shader selection

### Target Architecture
```toml
# Example pipeline template
[pipeline.forward]
name = "Forward Rendering"
vertex_shader = "shaders/forward.vert"
fragment_shader = "shaders/forward.frag"
```

### Tasks
- [ ] Define pipeline template format (TOML)
- [ ] Create pipeline template loader
- [ ] Refactor backends to load shaders from templates
- [ ] Remove hardcoded shader paths
- [ ] Add shader hot-reloading capability

### Deliverables
- Pipeline template specification
- Template loader implementation
- Dynamic shader loading in backends
- Example pipeline templates

## Phase 3: Remove Hardcoded Pipeline State

**Goal:** All pipeline state defined in templates

### Current State
- Depth testing: hardcoded enable/disable
- Backface culling: hardcoded settings
- Blend modes: hardcoded
- Primitive topology: hardcoded triangles
- Viewport/scissor: hardcoded to window size

### Target Architecture
```toml
[pipeline.forward]
name = "Forward Rendering"

[pipeline.forward.depth]
test = true
write = true
compare_op = "Less"

[pipeline.forward.rasterization]
cull_mode = "Back"
front_face = "CounterClockwise"
polygon_mode = "Fill"

[pipeline.forward.blending]
enabled = false
```

### Tasks
- [ ] Extend pipeline template format for state
- [ ] Implement state parser
- [ ] Refactor backends to use template state
- [ ] Remove hardcoded state from backends
- [ ] Add state validation

## Phase 4: Remove Hardcoded Resource Bindings

**Goal:** Descriptor sets/layouts defined by templates

### Current State
- Descriptor set layouts hardcoded in backends
- Binding numbers hardcoded (set 0 binding 0 = camera, etc.)
- Push constant layouts hardcoded
- No flexibility for different shaders

### Target Architecture
```toml
[pipeline.forward.bindings]
# Set 0: Per-frame uniforms
[[pipeline.forward.bindings.sets]]
set = 0
bindings = [
  { binding = 0, type = "UniformBuffer", stage = "Vertex" },  # Camera
  { binding = 1, type = "UniformBuffer", stage = "Fragment" },  # Lighting
  { binding = 2, type = "Texture", stage = "Fragment" },  # Base color
  { binding = 3, type = "UniformBuffer", stage = "Fragment" },  # Material
  { binding = 4, type = "Sampler", stage = "Fragment" }
]

# Set 1: Per-object data  
[[pipeline.forward.bindings.sets]]
set = 1
bindings = [
  { binding = 0, type = "PushConstant", stage = "Vertex", size = 128 }  # Transform
]
```

### Tasks
- [ ] Define binding template format
- [ ] Implement binding template parser
- [ ] Create dynamic descriptor layout builder
- [ ] Refactor backends to use template bindings
- [ ] Remove hardcoded layouts

## Phase 5: Flexible Vertex Formats

**Goal:** Support multiple vertex formats per pipeline

### Current State
- Single hardcoded vertex format: `[position, normal, uv, color]`
- All geometry must match this format
- No support for custom attributes

### Target Architecture
```toml
[pipeline.forward.vertex_input]
[[pipeline.forward.vertex_input.attributes]]
location = 0
format = "Float3"  # position

[[pipeline.forward.vertex_input.attributes]]
location = 1
format = "Float3"  # normal

[[pipeline.forward.vertex_input.attributes]]
location = 2
format = "Float2"  # uv
```

### Tasks
- [ ] Define vertex format in templates
- [ ] Implement format parser and validator
- [ ] Create dynamic vertex input builder
- [ ] Support multiple formats per application
- [ ] Validate geometry against format

## Phase 6: Scene-Driven Rendering

**Goal:** Everything comes from scene files + GLTF

### Current State
- Basic scene file support
- GLTF loading implemented
- Some hardcoded default values

### Target Architecture
- Scene file specifies which pipeline to use
- GLTF provides geometry, materials, textures
- No defaults in code, all in scene file

```toml
[scene]
name = "Test Cube"
pipeline = "forward"  # References pipeline template

[[scene.objects]]
mesh = "assets/models/cube.gltf#Cube"
material = "assets/materials/checkerboard.toml"
transform = { position = [0, 0, -5] }
```

### Tasks
- [ ] Link scene to pipeline template
- [ ] Remove default material/texture creation
- [ ] Require all data from files
- [ ] Add scene validation
- [ ] Error if data missing

## Phase 7: Complete Data-Driven Validation

**Goal:** Zero hardcoded rendering logic in backends

### Acceptance Criteria
- [ ] Backends contain zero shader paths
- [ ] Backends contain zero pipeline state values
- [ ] Backends contain zero descriptor layouts
- [ ] All rendering driven by templates + scenes
- [ ] CI validates this (grep for hardcoded values)

### Validation Tests
- [ ] CI scans backend code for hardcoded values
- [ ] Attempt to render with minimal/missing templates (should fail gracefully)
- [ ] Swap pipeline templates at runtime
- [ ] Hot-reload pipeline templates

## Success Metrics

### Technical
- Zero hardcoded values in backend implementations
- All rendering configurable via files
- Shader hot-reloading working
- Backend parity maintained (Vulkan ≈ DirectX)

### Process
- CI catches visual regressions automatically
- CI validates backend parity
- Easy to add new pipelines without touching backend code
- Easy to add new backends (just implement template interpreter)

## Timeline Estimate

| Phase | Estimated Time |
|-------|----------------|
| Phase 1: CI Rendering | 2-3 days |
| Phase 2: Shader Templates | 2-3 days |
| Phase 3: State Templates | 3-4 days |
| Phase 4: Binding Templates | 4-5 days |
| Phase 5: Vertex Formats | 2-3 days |
| Phase 6: Scene-Driven | 2-3 days |
| Phase 7: Validation | 1-2 days |
| **Total** | **16-23 days** |

## Implementation Order

1. **Phase 1 (CI)** - Most important, prevents regressions
2. **Phase 2 (Shaders)** - Foundation for templates
3. **Phase 3 (State)** - Second most visible hardcoding
4. **Phase 4 (Bindings)** - Most complex refactor
5. **Phase 5 (Vertex)** - Nice to have, less critical
6. **Phase 6 (Scene)** - Polish
7. **Phase 7 (Validation)** - Verification

## Next Steps

**Immediate (This Session):**
1. Update this plan based on feedback
2. Update ROADMAP.md to reflect these phases
3. Create GitHub issues for Phase 1
4. Start Phase 1: CI Rendering implementation

**This Week:**
- Complete Phase 1: CI rendering and comparison
- Begin Phase 2: Shader template design

**This Month:**
- Complete Phases 2-4
- Have data-driven shaders, state, and bindings working

## References

- See `ARCHITECTURE_CLEANUP_ROADMAP.md` for previous roadmap
- See `.github/workflows/ci.yml` for current CI setup
- See `scripts/generate_visual_report.py` for FLIP comparison
- See `docs/DESIGN.md` for overall architecture

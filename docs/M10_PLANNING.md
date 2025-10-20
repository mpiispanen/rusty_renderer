# M10: Unified Application & Scene-Driven Rendering

**Status:** 🚧 IN PROGRESS (Phase 0 Complete)  
**Started:** 2025-10-20  
**Target:** Complete unified application with scene-based rendering  

---

## Overview

M10 transforms the renderer from hardcoded examples to a unified, scene-driven application. Users can define scenes in TOML files and select rendering pipelines from the command line.

**Key Change:** From "multiple example programs" → "one application + scene files"

## Phases

### Phase 0: Foundation ✅ COMPLETE

**Goal:** Establish core architecture without full integration

**Completed:**
1. **Scene System**
   - TOML scene file format
   - Scene loading and validation
   - Support for inline geometry
   - Camera and light definitions
   - Transform system

2. **Pipeline Template System**
   - `RenderPipeline` trait
   - `PipelineFactory` for discovery
   - `SimplePipeline` implementation
   - Pipeline listing

3. **Unified Application**
   - Command-line interface (clap)
   - `ApplicationRunner` lifecycle
   - Scene and pipeline selection
   - Headless/interactive modes

**Status:** Foundation complete, ready for integration

**Files Added:** 13 (scene system, pipelines, application framework)  
**Tests:** +11 (108 total passing)  
**Documentation:** M10_PHASE0_COMPLETE.md

---

### Phase 1: Integration (NEXT) 🎯

**Goal:** Connect all pieces and enable actual rendering

**Tasks:**

1. **Complete ApplicationRunner Integration**
   - Initialize backend based on CLI args
   - Call `pipeline.setup(backend)`
   - Call `pipeline.build_graph(scene, backend)`
   - Compile and execute render graph
   - Handle cleanup properly

2. **Event Loop Implementation**
   - Interactive mode: proper window event loop
   - Headless mode: render single frame
   - Frame limiting (--max-frames)
   - Screenshot capture (--screenshot)

3. **SimplePipeline Implementation**
   - Build actual render graph from scene
   - Create vertex buffers from scene geometry
   - Add render passes to graph
   - Handle transforms (MVP matrices)
   - Support multiple objects

4. **Example Updates**
   - Update `render_graph_triangle` to use new system
   - Update `vertex_buffer_triangle` to use new system
   - Create compatibility wrapper if needed

5. **CI Fixes**
   - Re-enable GPU rendering tests
   - Fix DirectX test failures
   - Verify visual output matches

**Estimated Time:** 3-4 hours

**Acceptance Criteria:**
- [ ] Can render triangle scene: `cargo run -- --scene scenes/triangle.toml`
- [ ] Can render quad scene: `cargo run -- --scene scenes/quad.toml`
- [ ] Headless mode works: `cargo run -- --scene scenes/triangle.toml --headless`
- [ ] Screenshot works: `cargo run -- --scene scenes/triangle.toml --screenshot out.png`
- [ ] All backends work (Vulkan, wgpu, DirectX)
- [ ] GPU tests pass in CI
- [ ] Visual output matches M9 examples

---

### Phase 2: Camera System (TODO)

**Goal:** Interactive camera control

**Tasks:**
1. **Camera Controller**
   - Free-fly camera controls
   - Keyboard input (WASD + QE)
   - Mouse look
   - Configuration from scene file

2. **View/Projection Matrices**
   - Perspective projection
   - View matrix from camera transform
   - MVP matrix computation
   - Pass to shaders via uniform buffer

3. **Scene Updates**
   - Add camera parameters to scene files
   - Test with different camera positions
   - Verify FOV, near/far plane work

**Estimated Time:** 2-3 hours

**Acceptance Criteria:**
- [ ] Camera controller implemented
- [ ] Can move camera with WASD
- [ ] Can look around with mouse
- [ ] Scene file specifies camera
- [ ] Works with all backends

---

### Phase 3: Forward Rendering Pipeline (TODO)

**Goal:** Lit rendering with textures

**Tasks:**
1. **Lighting System**
   - Directional light support
   - Point light support
   - Light data in uniform buffer
   - Phong/Blinn-Phong shading

2. **ForwardPipeline Implementation**
   - Multi-pass rendering
   - Depth testing
   - Lighting calculations
   - Material system

3. **Shader Updates**
   - Lighting vertex/fragment shaders
   - Normal transformation
   - Material properties

**Estimated Time:** 4-5 hours

**Acceptance Criteria:**
- [ ] ForwardPipeline implemented
- [ ] Directional light works
- [ ] Point light works
- [ ] Proper depth testing
- [ ] Multiple objects render correctly

---

### Phase 4: Material & Texture System (TODO)

**Goal:** Textured, material-driven rendering

**Tasks:**
1. **Material System**
   - Material definitions in scenes
   - Albedo, metallic, roughness
   - Normal maps
   - Material loading

2. **Texture Integration**
   - Texture loading from files
   - Texture binding in pipelines
   - Sampler configuration
   - UV coordinate support

3. **Enhanced Scenes**
   - Textured cube scene
   - Multi-material objects
   - Test different material properties

**Estimated Time:** 3-4 hours

**Acceptance Criteria:**
- [ ] Materials defined in scene files
- [ ] Textures load from disk
- [ ] Textured objects render
- [ ] Normal maps work
- [ ] Multiple materials per scene

---

## Architecture

### Before M10
```
examples/
  triangle.rs          // Hardcoded triangle
  quad.rs              // Hardcoded quad
  vertex_buffer_triangle.rs  // Hardcoded with buffers
```

Each example:
- Manually creates geometry
- Manually builds render graph
- Duplicates setup code

### After M10
```
src/
  scene/              // Scene definition & loading
  pipelines/          // Rendering strategies
  application/        // Unified entry point

scenes/
  triangle.toml       // Scene definition
  quad.toml
  my_custom.toml
  
$ cargo run -- --scene scenes/triangle.toml --pipeline simple
$ cargo run -- --scene my_scene.toml --pipeline forward
```

One application, many scenes.

### Component Interaction

```
┌─────────────────────────────────────────────────────────┐
│                   ApplicationRunner                      │
│  • Parses CLI args                                      │
│  • Loads scene from TOML                                │
│  • Creates pipeline                                     │
│  • Initializes backend                                  │
│  • Runs event loop                                      │
└────────────┬────────────────────────────────────────────┘
             │
       ┌─────┴──────┐
       │            │
       ▼            ▼
┌─────────────┐  ┌──────────────┐
│    Scene    │  │   Pipeline   │
│             │  │              │
│ • Objects   │  │ • setup()    │
│ • Camera    │  │ • build_     │
│ • Lights    │  │   graph()    │
│ • Metadata  │  │ • cleanup()  │
└──────┬──────┘  └──────┬───────┘
       │                │
       └────────┬───────┘
                │
                ▼
         ┌─────────────┐
         │RenderGraph  │
         │  • Passes   │
         │  • Resources│
         │  • Compiled │
         └──────┬──────┘
                │
                ▼
         ┌─────────────┐
         │   Backend   │
         │  • Vulkan   │
         │  • wgpu     │
         │  • DirectX  │
         └─────────────┘
```

## Technical Decisions

### Scene Format: TOML
**Rationale:**
- Human-readable and writable
- Good Rust ecosystem support (serde)
- Clear structure for hierarchies
- Better than JSON (less verbose)
- Better than RON (more familiar)

**Example:**
```toml
[metadata]
name = "RGB Triangle"
description = "Simple colored triangle"

[[objects]]
type = "mesh"

[objects.geometry]
type = "inline"
vertices = [
    { position = [0.0, -0.5, 0.0], color = [1.0, 0.0, 0.0] },
    { position = [0.5, 0.5, 0.0], color = [0.0, 1.0, 0.0] },
    { position = [-0.5, 0.5, 0.0], color = [0.0, 0.0, 1.0] },
]
```

### Pipeline as Trait
**Rationale:**
- Multiple rendering strategies
- Easy to add new pipelines
- Clear separation of concerns
- Testable independently

**Pipelines:**
- `SimplePipeline` - Vertex colors only
- `ForwardPipeline` - Lighting + textures
- `DeferredPipeline` - Future
- `DebugPipeline` - Wireframe, normals, etc.

### Gradual Integration
**Rationale:**
- Lower risk
- Each piece tested independently
- Clear phases and goals
- Can pause/resume easily

**Phases:**
0. Foundation (structure)
1. Integration (connect pieces)
2. Camera (interactivity)
3. Forward (lighting)
4. Materials (textures)

## Testing Strategy

### Unit Tests
- Scene loading and validation
- Pipeline factory
- Application argument parsing
- Transform calculations

### Integration Tests  
- Scene → Pipeline → RenderGraph
- Backend initialization
- Full render loop

### Visual Tests
- Headless rendering
- Screenshot comparison
- Reference images
- CI validation

### Manual Tests
- Interactive mode
- Camera controls
- Different scenes
- All backends

## Success Criteria

### M10 Phase 0 ✅
- [x] Scene system implemented
- [x] Pipeline system implemented
- [x] Application framework implemented
- [x] All unit tests pass
- [x] CLI works correctly

### M10 Phase 1 (Next)
- [ ] Scene renders correctly
- [ ] All backends work
- [ ] Headless mode works
- [ ] Screenshots work
- [ ] GPU tests pass in CI

### M10 Complete
- [ ] All 4 phases done
- [ ] Camera control works
- [ ] Forward rendering works
- [ ] Materials and textures work
- [ ] Multiple example scenes
- [ ] Full CI coverage
- [ ] Documentation complete

## Known Issues & Deferred Work

### From Phase 0
1. **GPU Test Failures** (Expected)
   - Old examples not updated yet
   - Will fix in Phase 1
   - CI: Test (GPU - Render Graph Examples)
   - CI: Build (Windows + DirectX 12)

2. **SimplePipeline Incomplete**
   - Currently just logs
   - Needs full render graph building
   - Phase 1 task

3. **No Backend Initialization**
   - ApplicationRunner doesn't init backend yet
   - Phase 1 task

4. **No Event Loop**
   - Just exits after initialization
   - Phase 1 task

### Future Work (After M10)
- glTF model loading (M11)
- Multiple objects per pass (batching)
- Indexed geometry support
- External geometry files
- Scene graph/hierarchy
- Advanced materials (PBR)
- Shadow mapping
- Post-processing

## Dependencies

### Before M10 (Complete)
- ✅ M8: Resource Manager
- ✅ M9: Render Graph Execution

### For M10
- ✅ Scene loader (Phase 0)
- ✅ Pipeline system (Phase 0)
- ✅ Application framework (Phase 0)
- 🚧 Full integration (Phase 1)
- ⏳ Camera system (Phase 2)
- ⏳ Forward renderer (Phase 3)
- ⏳ Materials (Phase 4)

### After M10
- ⏳ M11: glTF Loading
- ⏳ M12: Advanced Rendering

## Timeline

- **Phase 0:** ✅ Complete (2025-10-20, ~2 hours)
- **Phase 1:** 🎯 Next session (~3-4 hours)
- **Phase 2:** Future (~2-3 hours)
- **Phase 3:** Future (~4-5 hours)
- **Phase 4:** Future (~3-4 hours)

**Total Estimated:** ~14-18 hours for complete M10

## References

- M10_PHASE0_COMPLETE.md - Phase 0 summary
- M9_PLANNING.md - Previous milestone planning
- M9_RETROSPECTIVE.md - Lessons learned
- docs/DESIGN.md - Overall design document

## Notes

### Why This Matters

This milestone is crucial because:
1. **User Experience:** Makes renderer actually usable
2. **Maintainability:** Reduces code duplication
3. **Flexibility:** Easy to add new scenes/pipelines
4. **Testing:** Can automate visual validation
5. **Future:** Foundation for glTF, PBR, etc.

### Design Philosophy

- **Scene = What:** Objects, lights, camera, materials
- **Pipeline = How:** Rendering strategy, passes, shaders
- **Application = When:** CLI, events, lifecycle

Clean separation of concerns enables:
- Reusable scenes across pipelines
- Reusable pipelines across scenes
- Easy testing and validation
- Clear mental model

---

**Status:** Phase 0 complete, Phase 1 next  
**Last Updated:** 2025-10-20  
**Next Session:** Phase 1 integration

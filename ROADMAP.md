# Rusty Renderer - Development Roadmap

## ✅ Completed Features

### Core Architecture
- [x] Multi-backend abstraction layer
- [x] Vulkan backend (fully functional)
- [x] DirectX 12 backend (functional)
- [x] Window management (winit integration)
- [x] Event loop handling

### Scene System
- [x] TOML-based scene files
- [x] Scene loader with validation
- [x] Camera system (perspective)
- [x] Lighting system (directional, point lights)
- [x] Material system (PBR properties)
- [x] Transform system (position, rotation, scale)

### Asset Management
- [x] Asset path resolution (flexible, portable)
- [x] Texture loading (PNG support via image crate)
- [x] GLTF model loading
- [x] Material texture support
- [x] Proper project structure (assets directory)

### Rendering Pipelines
- [x] Simple pipeline (vertex colors)
- [x] Forward pipeline (lighting + textures)
- [x] Pipeline factory pattern
- [x] Render graph system

### Build System
- [x] Shader compilation (GLSL to SPIR-V)
- [x] Shader validation
- [x] Cross-platform builds
- [x] Test infrastructure

### Documentation
- [x] Architecture documentation
- [x] Asset system guide
- [x] Scene file format documentation
- [x] Implementation summaries

## 🔄 In Progress (Current Focus)

### Phase 1: CI Rendering & Visual Regression (CURRENT)
- [ ] Update CI to render test scenes on Vulkan and DirectX
- [ ] Enable headless rendering in CI
- [ ] Automated backend parity validation with FLIP
- [ ] Golden reference image library
- [ ] Visual regression reports

**Why This First:** Prevents regressions as we refactor, ensures backends stay in sync

## 📋 TODO - High Priority (Data-Driven Architecture)

See `ARCHITECTURE_REFACTOR_PLAN.md` for detailed plan.

### Phase 2: Remove Hardcoded Shaders (Next)
- [ ] Design pipeline template format (TOML)
- [ ] Implement template loader
- [ ] Dynamic shader loading from templates
- [ ] Remove hardcoded shader paths from backends
- [ ] Shader hot-reloading

### Phase 3: Remove Hardcoded Pipeline State
- [ ] Define state in pipeline templates (depth, culling, blending)
- [ ] Implement state parser
- [ ] Refactor backends to use template state
- [ ] Remove all hardcoded pipeline state

### Phase 4: Remove Hardcoded Resource Bindings
- [ ] Define descriptor layouts in templates
- [ ] Dynamic descriptor layout builder
- [ ] Remove hardcoded binding numbers
- [ ] Support flexible binding configurations

### Phase 5: Flexible Vertex Formats
- [ ] Define vertex formats in templates
- [ ] Support multiple formats per application
- [ ] Validate geometry against format
- [ ] Remove single hardcoded format

### Phase 6: Complete Scene-Driven Rendering
- [ ] Link scenes to pipeline templates
- [ ] Remove all default materials/textures from code
- [ ] All data must come from files
- [ ] Graceful error handling for missing data

### Phase 7: Validation
- [ ] Zero hardcoded values in backends (automated check)
- [ ] Runtime pipeline swapping
- [ ] Hot-reload validation
- [ ] Complete data-driven architecture verified

## 📋 Completed Features

### Backend Development
- [x] ✅ DirectX 12 Backend Complete
- [x] ✅ Vulkan Backend Complete  
- [x] ✅ Both backends achieve visual parity
- [x] ✅ Backface culling fixed
- [x] ✅ Depth testing working
- [x] ✅ Coordinate system differences handled
- [x] ✅ Cross-compilation for Windows
- [x] ✅ Proton testing on Linux

### Scene & Asset System
- [x] ✅ GLTF Loading
- [x] ✅ Texture loading and sampling
- [x] ✅ Scene system with TOML
- [x] ✅ Forward rendering with lighting
- [x] ✅ Material system

## 📋 TODO - Medium Priority (After Architecture Refactor)

### Enhanced Rendering Features
- [ ] Deferred rendering pipeline
- [ ] Shadow mapping
- [ ] Post-processing effects  
- [ ] Multi-pass rendering
- [ ] Render to texture
- [ ] Additional texture maps:
  - [ ] Normal maps
  - [ ] Metallic/roughness maps
  - [ ] Ambient occlusion
  - [ ] Emissive maps

## 📋 TODO - Medium Priority

### Asset System Enhancements
- [ ] Asset caching system
- [ ] Hot reloading
- [ ] Asset preprocessing
- [ ] Packaged asset format
- [ ] Asset compression
- [ ] LOD support

### Scene System Improvements
- [ ] Scene hierarchy (parent/child transforms)
- [ ] Multiple cameras
- [ ] Camera switching
- [ ] Scene composition (load multiple scenes)
- [ ] External geometry file support (OBJ, FBX)

### Material System
- [ ] Material editor
- [ ] Shader graph system
- [ ] Custom material properties
- [ ] Material instancing
- [ ] Material hot reload

### Animation System
- [ ] Skeletal animation
- [ ] Morph targets/blend shapes
- [ ] Animation blending
- [ ] GLTF animation support
- [ ] Timeline editor

## 📋 TODO - Lower Priority

### Advanced Rendering
- [ ] PBR improvements
- [ ] Image-based lighting (IBL)
- [ ] Global illumination
- [ ] Ray tracing support
- [ ] Volumetric effects
- [ ] Particle systems

### Performance
- [ ] Frustum culling
- [ ] Occlusion culling
- [ ] Level of detail (LOD)
- [ ] Instancing
- [ ] Multi-threading
- [ ] GPU profiling

### Tools & Editor
- [ ] Visual scene editor
- [ ] Material editor GUI
- [ ] Performance profiler
- [ ] Asset browser
- [ ] Shader debugger

### Platform Support
- [ ] Metal backend (macOS/iOS)
- [ ] Mobile optimization
- [ ] Web support (WebGPU)
- [ ] Console support

### Quality of Life
- [ ] Hot shader reloading
- [ ] Debug visualization
- [ ] Performance metrics overlay
- [ ] Screenshot system enhancement
- [ ] Video recording
- [ ] Benchmark suite

## 🎯 Current Sprint Focus

**Data-Driven Architecture Refactor** - Remove all hardcoded rendering

### Immediate Next Steps (This Week):
1. **Phase 1: CI Rendering** - Automated visual regression testing
   - Update CI workflows for headless rendering
   - Implement backend parity validation
   - Create golden reference images
   
2. **Phase 2: Shader Templates** - Remove hardcoded shaders
   - Design pipeline template format
   - Implement template loader
   - Dynamic shader loading

### Next Month:
3. **Phase 3-4: State & Bindings** - Complete template system
4. **Phase 5-7: Polish & Validation** - Verify data-driven architecture

## 📊 Progress Metrics

| Category | Complete | In Progress | TODO |
|----------|----------|-------------|------|
| Core Architecture | 95% | 5% (Data-driven refactor) | - |
| Backends (Vulkan/DX12) | 100% | - | - |
| Scene System | 85% | 5% | 10% |
| Asset Management | 80% | 5% | 15% |
| Data-Driven Pipeline | 0% | 10% | 90% |
| Rendering Pipelines | 40% | 10% | 50% |
| Advanced Features | 10% | 5% | 85% |
| CI/CD & Testing | 60% | 20% | 20% |

## 🔍 Known Issues

### Critical
- ⚠️ **Hardcoded rendering** - Shaders, pipeline state, bindings all hardcoded
- ⚠️ **No CI rendering** - Visual regressions not caught automatically
- ⚠️ **Slight color differences** - Vulkan vs DirectX minor color variations

### Major
- No embedded texture support in GLTF
- No shadow support
- No deferred rendering
- No animation support
- No pipeline templates
- No hot-reloading

### Minor
- Some unused variable warnings
- No asset caching
- Limited material features

## 💡 Design Decisions

### Why TOML for Scenes?
- Human-readable
- Easy to edit
- Good Rust support (serde)
- Suitable for configuration

### Why Multi-Backend?
- Platform flexibility
- Learning multiple APIs
- Fallback options
- Performance comparison

### Why Render Graph?
- Flexibility for complex pipelines
- Automatic resource management
- Easy to extend
- Industry standard approach

## 📚 Learning Resources Used

- Vulkan Tutorial
- DirectX 12 Documentation
- GLTF Specification
- PBR Theory

## 🎓 Skills Developed

- Graphics API abstraction
- Vulkan programming
- DirectX 12 programming
- GLSL shader programming
- Asset pipeline design
- Scene graph architecture
- Rust trait design patterns

---

**Last Updated**: 2025-10-27  
**Version**: 0.2.0  
**Status**: Active Development - Architecture Refactor Phase

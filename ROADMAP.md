# Rusty Renderer - Development Roadmap

**Last Updated:** 2025-11-03  
**Status:** Transitioning to Render Graph Driven Architecture

See [CURRENT_STATE.md](CURRENT_STATE.md) for detailed current status.

## ✅ Completed Features

### Core Architecture
- [x] Multi-backend abstraction layer
- [x] Vulkan backend (fully functional)
- [x] DirectX 12 backend (functional via Proton on Linux)
- [x] Window management (winit integration)
- [x] Event loop handling
- [x] Render graph system with resource management
- [x] Cross-compilation for Windows targets

### Scene System
- [x] TOML-based scene files
- [x] Scene loader with validation
- [x] Camera system (perspective)
- [x] Transform system (position, rotation, scale)
- [x] GLTF model loading with textures
- [x] Inline geometry support

### Rendering Features
- [x] Forward rendering with Blinn-Phong lighting
- [x] Simple triangle rendering (debug)
- [x] Depth testing and backface culling
- [x] Texture loading and sampling
- [x] Vertex color rendering
- [x] Headless and windowed modes
- [x] Screenshot capture

### Build System
- [x] Unified HLSL shader compilation (DXC)
- [x] SPIR-V generation for Vulkan
- [x] DXIL generation for DirectX
- [x] Shader validation
- [x] Cross-platform builds
- [x] Test infrastructure

### Documentation
- [x] Architecture documentation
- [x] Workflow and contribution guidelines
- [x] Bazzite/Linux setup guide
- [x] DirectX via Proton testing guide

## 🔄 In Progress (Current Sprint)

### Render Graph Architecture Completion (CURRENT)
- [x] Render pass resource declaration
- [x] Shader registry system
- [ ] **Remove legacy hardcoded code** (THIS WEEK)
  - [ ] Remove hardcoded vertices in app.rs
  - [ ] Move shader registration to render passes
  - [ ] Extract lights to scene files
  - [ ] Clean up unused code paths
- [ ] **Shadow mapping pipeline** (THIS WEEK)
  - [ ] Shadow map render pass
  - [ ] Forward pass with shadow support
  - [ ] Tone mapping post-process
- [ ] **CI rendering tests** (ONGOING)
  - [ ] Fix CI visual regression tests
  - [ ] Backend parity validation
  - [ ] Reference image management

**Why This Focus:** Complete the transition to render graph driven architecture before adding more features

## 📋 TODO - High Priority (Next Month)

### Shadow Mapping & Post-Processing
- [ ] Shadow map render pass (depth-only)
- [ ] Forward pass with shadow sampling
- [ ] PCF (Percentage Closer Filtering)
- [ ] Cascaded Shadow Maps (CSM)
- [ ] Tone mapping post-process pass

### Debug UI & Runtime Configuration
- [ ] ImGui integration
- [ ] Pipeline configuration UI
- [ ] Pass enable/disable toggles
- [ ] Real-time parameter adjustment
- [ ] Performance profiling overlay

### Scene System Improvements
- [ ] Extract lights to scene files (currently hardcoded in app)
- [ ] Multiple light types in scenes
- [ ] Scene hierarchy (parent/child transforms)
- [ ] Multiple cameras
- [ ] External geometry file support (OBJ)

### Material & Texture Enhancements
- [ ] Normal mapping
- [ ] Metallic/roughness maps
- [ ] Ambient occlusion maps
- [ ] Emissive maps
- [ ] Material hot-reload

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

**Render Graph Driven Architecture - Cleanup & Shadow Mapping**

### This Week - Code Cleanup & Shadows
1. ✅ **Remove Legacy Code**
   - Remove hardcoded vertices in app.rs
   - Move shader registration to render passes  
   - Extract lights to scene files
   - Clean up unused code paths

2. 🔄 **Shadow Mapping**
   - Implement shadow map render pass
   - Update forward pass for shadows
   - Add tone mapping post-process

3. 🔄 **CI Fixes**
   - Fix CI rendering tests
   - Backend parity validation
   - Reference image workflow

### Next Week - Debug UI
4. **ImGui Integration**
   - Basic UI framework
   - Pipeline configuration
   - Pass toggles
   - Performance metrics

### This Month - Enhanced Rendering
5. **Advanced Shadows**
   - PCF filtering
   - Cascaded shadow maps
   
6. **Post-Processing**
   - Tone mapping refinement
   - Additional effects (bloom, etc.)

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

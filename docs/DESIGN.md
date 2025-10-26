# Rusty Renderer - Design Document

**Version:** 0.6.0  
**Last Updated:** 2025-10-25  
**Status:** Architecture Refactor - Moving to Data-Driven System

## Project Vision

Rusty Renderer is a graphics rendering sandbox and experimentation engine built in Rust. The goal is to create a flexible, interactive environment for testing and developing different rendering algorithms and techniques. The engine prioritizes ease of experimentation, allowing developers to quickly iterate on rendering approaches, toggle render passes, visualize debug information, and profile performance.

## Core Principles

1. **Experimentation First**: Easy to test new rendering algorithms and techniques
2. **Multi-Backend Support**: First-class support for Vulkan and DirectX 12
3. **Interactive Development**: Live scene exploration, runtime shader compilation, debug visualization
4. **Render Graph Architecture**: Automatic dependency resolution and resource management
5. **Developer-Friendly**: Comprehensive debugging tools, profiling views, and hot-reloading

## Current State

**Phase:** Architecture Refactor - Data-Driven Rendering  
**Status:** DirectX/Vulkan backends functional, removing hardcoded rendering

### Recent Achievements (Oct 25, 2025)
- ✅ **DirectX Memory Management Fixed**: Proper heap types, staging buffers, resource states
- ✅ **DirectX Render Graph Working**: Full cube rendering with lighting
- ✅ **Backface Culling**: Enabled in DirectX pipeline
- ✅ **Legacy Code Removed**: No more hardcoded triangle in end_frame()
- ✅ **Normal-Based Debug Visualization**: Helps verify geometry without textures

### Completed (Production Ready on Vulkan + DirectX)
- ✅ Repository setup with CI/CD pipeline
- ✅ Project structure with proper module organization
- ✅ Backend abstraction layer with trait definitions
- ✅ Two backend implementations: Vulkan (complete), DirectX 12 (functional)
- ✅ Command-line argument parsing for backend selection
- ✅ Validation layer support (zero errors on Vulkan)
- ✅ Render graph architecture with automatic dependency resolution
- ✅ Render graph execution on all backends
- ✅ Modular pass system (passes in separate files)
- ✅ Cross-compilation setup for Windows targets
- ✅ DirectX 12 testing via Proton on Linux
- ✅ Headless rendering and screenshot capture
- ✅ Visual testing infrastructure (FLIP integration)
- ✅ **Scene system with TOML definitions**
- ✅ **Vertex and index buffer support**
- ✅ **Forward rendering with Blinn-Phong lighting**
- ✅ **Directional and point lights (up to 8)**
- ✅ **Per-object transforms via push constants**
- ✅ **Per-frame descriptor sets (proper synchronization)**
- ✅ **Camera system with view-projection matrices**
- ✅ **Normal transformation for non-uniform scaling**

### Working Features (Vulkan)
- Forward renderer with diffuse + specular lighting
- Multiple light types (directional, point)
- Per-object position, rotation, scale
- Ambient lighting
- Material colors
- Proper resource cleanup (0 validation errors)
- Headless and windowed modes

### Known Limitations & Roadmap

**Current Issues:**
- ❌ **DirectX: No depth testing** (requires depth buffer implementation)
- ❌ **DirectX: No texture support** (requires descriptor tables)
- ❌ **Hardcoded shaders**: Embedded HLSL triangle shader as fallback
- ❌ **Hardcoded vertex data**: Old triangle passes still exist
- ❌ **Pipeline state hardcoded**: Not driven by templates

**Architecture Goals (See `ARCHITECTURE_CLEANUP_ROADMAP.md`):**
1. **Phase 1: Backend Parity** - Vulkan/DirectX identical output
2. **Phase 2: Remove Hardcoding** - All data from files
3. **Phase 3: Pipeline Templates** - Rendering defined by TOML
4. **Phase 4: Scene-Driven** - Everything from glTF + scene files
5. **Phase 5: CI/CD** - Automated visual regression testing
6. **Phase 6: Validation** - Complete data-driven architecture

### In Progress (Current Focus)
- **Phase 1: Backend Parity** - DirectX depth testing, coordinate fixes
- **CI Setup**: Automated rendering comparisons between backends
- See `ARCHITECTURE_CLEANUP_ROADMAP.md` for complete plan

### Coordinate System Handling
All backends now properly handle coordinate system differences. Vulkan uses standard Y-down coordinates, while DirectX requires Y-axis flipping to maintain visual consistency. This is documented in `docs/COORDINATE_SYSTEMS.md`.

## Architecture Overview

### High-Level Structure

**Directory Layout:**

- **src/** - Main source code
  - **main.rs** - Application entry point
  - **app.rs** - Main application loop and state
  - **backends/** - Graphics backend implementations
    - **mod.rs** - Backend trait definitions
    - **vulkan.rs** - Vulkan implementation (vulkanalia)
    - **directx.rs** - DirectX 12 implementation
  - **render_graph/** - Render graph system
    - **mod.rs** - Core render graph
    - **pass.rs** - Render pass abstraction
    - **resource.rs** - Resource management
    - **scheduler.rs** - Execution scheduling
  - **scene/** - Scene representation
    - **mod.rs** - Scene module root
    - **camera.rs** - Camera systems
    - **mesh.rs** - Mesh abstraction
    - **loader.rs** - Asset loading (glTF, etc.)
  - **shaders/** - Shader management
    - **mod.rs** - Shader module root
    - **compiler.rs** - Runtime shader compilation
    - **cache.rs** - Shader caching
  - **ui/** - Debug UI
    - **mod.rs** - UI module root
    - **debug_panel.rs** - Debug visualization panels
  - **profiling/** - Performance profiling
    - **mod.rs** - Profiling module root
    - **metrics.rs** - Performance metrics collection
- **tests/** - Integration tests
- **shaders/** - Shader source files
- **assets/** - Test assets and scenes

### Core Components

#### 1. Backend Abstraction Layer

**Design Goal**: Provide a unified trait-based interface that abstracts over Vulkan and DirectX 12.

**Key Traits**:
- `GraphicsBackend`: Main backend interface
- `Device`: Device creation and management
- `CommandBuffer`: Command recording
- `Pipeline`: Graphics pipeline abstraction
- `Resource`: Buffers, textures, and other GPU resources
- `Swapchain`: Presentation surface management

**Implementation Strategy**:
- Vulkan (primary) → DirectX 12
- Each backend as a module within the same crate
- Shared validation and error handling
- Backend-specific optimizations isolated within implementations

#### 2. Render Graph System

**Design Goal**: Runtime render graph that automatically handles resource dependencies, execution ordering, and synchronization.

**Core Concepts**:
- **Render Pass**: Coarse-grained rendering operation (e.g., "Shadow Pass", "GBuffer Pass", "Lighting Pass")
- **Resource Declaration**: Passes explicitly declare input/output resources
- **Automatic Scheduling**: Graph determines execution order based on dependencies
- **Automatic Barriers**: Resource transitions and synchronization handled automatically

**Resource Management:**

- Explicit resource lifetime tracking
- Automatic barrier insertion between passes
- Transient resource optimization (future enhancement - aliasing/pooling)

**Graph Structure Example:**

```rust
// Conceptual API (to be refined)
let mut graph = RenderGraph::new();

graph.add_pass("shadow_pass")
    .writes(shadow_map)
    .execute(|ctx| { /* render shadows */ });

graph.add_pass("gbuffer_pass")
    .writes(albedo_texture)
    .writes(normal_texture)
    .writes(depth_texture)
    .execute(|ctx| { /* render gbuffer */ });

graph.add_pass("lighting_pass")
    .reads(shadow_map)
    .reads(albedo_texture)
    .reads(normal_texture)
    .reads(depth_texture)
    .writes(final_image)
    .execute(|ctx| { /* compute lighting */ });

graph.compile_and_execute(backend);
```

#### 3. Scene System

**Design Goal**: Flexible scene representation with glTF as primary format, abstracted for future expansion.

**Components:**

- **Scene Graph:** Hierarchical scene structure
- **Asset Loader:** glTF loading with custom abstraction layer
- **Camera System:** Free-fly camera for scene exploration
- **Mesh Representation:** Backend-agnostic mesh data

**Asset Pipeline:**

- glTF as primary format
- Custom intermediate representation
- Future support for additional formats

#### 4. Shader Management

**Design Goal:** Support both online (runtime) and offline (pre-compiled) shader workflows.

**Features:**

- Runtime compilation (GLSL/HLSL → SPIR-V)
- Shader hot-reloading for development
- Shader caching to avoid recompilation
- Offline compilation path for production
- Cross-compilation for different backends

**Tools:**

- `shaderc` or `naga` for runtime compilation
- SPIR-V as intermediate format
- Backend-specific translation where needed

#### 5. Debug UI and Visualization

**Design Goal:** Comprehensive debugging and profiling interface using egui.

**Features:**

- Render pass toggle (enable/disable individual passes)
- Debug view selection (normals, depth, overdraw, etc.)
- Performance metrics display
- Resource inspector
- Shader reload controls
- Scene hierarchy viewer

**Debug Views (Progressive Implementation):**

- Render pass outputs
- Depth buffer visualization
- Normal buffer visualization
- Wireframe mode
- Overdraw heatmap
- Light visualization
- Shadow map inspection

#### 6. Profiling System

**Design Goal:** Multi-faceted performance analysis for both CPU and GPU.

**Metrics (To be detailed in later phases):**

- GPU timestamps per render pass
- CPU frame time breakdown
- Memory usage tracking
- Draw call statistics
- Resource binding costs

### Application Framework

- **Window Management:** winit for cross-platform windowing
- **Input Handling:** Deferred until basic rendering is functional
- **Application Structure:** Monolithic engine with modular components
- **Configuration:** Command-line argument parsing (clap or similar)

## Development Roadmap

### Short Term (Immediate - ~2-4 weeks)

**Milestone 1: Project Foundation** ✅ COMPLETED
- [x] Set up Cargo project structure with proper module organization
- [x] Implement command-line argument parsing (backend selection, window size, etc.)
- [x] Create basic application loop with winit
- [x] Set up CI/CD pipeline on GitHub with local runner for graphics tests
- [x] Implement basic unit test framework

**Milestone 2: Backend Abstraction - Stub Implementation** ✅ COMPLETED
- [x] Define core backend traits (`GraphicsBackend`, `Device`, `CommandBuffer`, etc.)
- [x] Create stub implementations for backends (Vulkan, DirectX)
- [x] Implement backend selection and initialization
- [x] Unit tests for backend trait contracts

**Milestone 3: Vulkan Triangle** ✅ COMPLETED
- [x] Implement Vulkan backend (vulkanalia) for basic rendering
- [x] Create simple hardcoded vertex buffer (triangle data)
- [x] Implement basic shader loading (hardcoded simple vertex/fragment shaders)
- [x] Render a single triangle to screen
- [x] Integration test: verify triangle renders correctly
- [x] Add validation layer support

**Milestone 4: Multi-Backend Triangle** ✅ COMPLETED
- [x] Implement DirectX 12 backend for triangle rendering
- [x] Test DirectX implementation on Linux via Proton
- [x] Handle coordinate system differences across backends
- [x] Integration tests for both backends
- [x] Cross-compilation setup for Windows

**Milestone 5: Infrastructure and Testing** (IN PROGRESS)
- [ ] Offscreen rendering mode for CI testing without window display
- [ ] Screenshot functionality for visual validation
- [ ] Visual correctness testing between backends
- [ ] Golden reference image comparison system
- [ ] Git LFS setup for test images
- [ ] Documentation organization (move docs to docs/)
- [ ] Keep design document updated with progress

### Medium Term (~1-2 months)

**Future Milestone: Render Graph Foundation**
- [ ] Design and implement core render graph data structures
- [ ] Implement pass registration and resource declaration
- [ ] Build dependency resolution and topological sorting
- [ ] Automatic barrier insertion for resource transitions
- [ ] Refactor triangle demo to use render graph
- [ ] Unit tests for render graph scheduling and validation

**Future Milestone: Enhanced Graphics Pipeline**
- [ ] Abstract vertex/index buffer management
- [ ] Implement uniform buffer abstraction
- [ ] Support for multiple shader stages
- [ ] Texture and sampler abstraction
- [ ] Render multiple objects with different materials

**Future Milestone: Basic Scene System**
- [ ] Implement scene graph structure
- [ ] Create simple glTF loader (using gltf crate)
- [ ] Mesh abstraction and GPU upload
- [ ] Transform hierarchy
- [ ] Load and render a basic glTF model

**Future Milestone: Camera and Controls**
- [ ] Free-fly camera implementation
- [ ] Input handling integration (keyboard/mouse)
- [ ] Camera movement and rotation
- [ ] View/projection matrix management

**Future Milestone: Debug UI Integration**
- [ ] Integrate egui into the application
- [ ] Basic debug panel with FPS counter
- [ ] Render pass enable/disable toggles
- [ ] Camera position/orientation display
- [ ] Render graph visualization

**Future Milestone: Shader Hot-Reloading**
- [ ] Online shader compilation (runtime GLSL/HLSL → SPIR-V)
- [ ] File watching for shader changes
- [ ] Hot-reload pipeline without application restart
- [ ] Error reporting in debug UI
- [ ] Shader editor integration

### Long Term (3+ months)

**Advanced Rendering Features**
- Physically-Based Rendering (PBR) materials
- Shadow mapping (directional, point, spot lights)
- Deferred rendering pipeline
- Screen-space ambient occlusion (SSAO)
- Tone mapping and HDR rendering
- Post-processing effects (bloom, depth of field, etc.)

**Advanced Debug and Profiling**
- GPU profiler integration (RenderDoc, PIX, etc.)
- Advanced debug views (overdraw, shader complexity heatmap)
- Memory profiling and leak detection
- Frame capture and replay

**Scene and Asset Management**
- Advanced glTF features (skinning, animations, morph targets)
- Material system with parameter editing
- Asset hot-reloading (models, textures)
- Scene serialization/deserialization

**Render Graph Enhancements**
- Transient resource aliasing and memory pooling
- Multi-threaded command buffer recording
- GPU-driven rendering support
- Render graph visualization in debug UI
- Frame graph debugging and inspection

**Quality of Life**
- Configuration file support
- Multiple scene loading
- Screenshot/frame capture
- Render preset system

## Technical Decisions

### Language and Tooling
- **Language**: Rust (latest stable)
- **Build System**: Cargo
- **MSRV**: Rust 1.75+ (or latest stable at project start)

### Key Dependencies
- **Graphics APIs**:
  - `vulkanalia` - Vulkan bindings
  - `windows-rs` - DirectX 12 bindings
- **Windowing**: `winit`
- **UI**: `egui` with appropriate backend integration
- **Math**: `glam` (or `nalgebra`)
- **Asset Loading**: `gltf` for glTF parsing
- **Shader Compilation**: `shaderc` or `naga`
- **Argument Parsing**: `clap`

### Development Workflow

1. **Issue-Driven Development**
   - Create detailed issues for planned work
   - Issues should include acceptance criteria and test requirements
   - Reference design document in issues

2. **Implementation Process**
   - Define what to implement (issue/design)
   - Write tests (unit and/or integration)
   - Implement feature to pass tests
   - Update design document if architecture changes

3. **Testing Strategy**
   - Unit tests for individual components
   - Integration tests for end-to-end functionality
   - Graphics validation tests on local CI runner
   - Manual testing for visual validation

4. **Backend Implementation Order**
   - Implement feature in Vulkan first (primary development platform)
   - Validate design works for DirectX (test on Linux via Proton)
   - Ensure both backends achieve feature parity
   - Avoid over-investing in single backend before validating across APIs

5. **Documentation Maintenance**
   - Keep DESIGN.md updated as architecture evolves
   - Move documentation files to docs/ directory (avoid root clutter)
   - Create retrospectives after each milestone
   - Always ensure CI passes before closing issues

## Open Questions and Future Considerations

### Render Graph
### Render Graph

- **Compile-time vs Runtime:** Currently runtime, but evaluate hybrid approach
- **Transient Resource Optimization:** Aliasing and pooling strategy TBD
- **Multi-threading:** Command buffer recording parallelization strategy

### Performance

- **Memory Allocators:** Custom GPU memory allocator vs. library (gpu-allocator)
- **Command Buffer Pooling:** Recycling strategy
- **Descriptor Set Management:** Bindless vs. traditional

### Platform Support

- **Console Platforms:** Out of initial scope, but consider in design

### Extensibility

- **Plugin System:** Future consideration for custom render passes
- **Scripting:** Hot-reloadable logic for rapid iteration

## Success Criteria

### Short Term Success
- Triangle rendering on both backends (Vulkan, DirectX)
- Basic application structure with proper error handling
- CI/CD pipeline operational
- Unit and integration tests passing

### Medium Term Success
- Render graph system functional with automatic dependency management
- Load and render glTF models
- Interactive camera controls
- Debug UI with render pass controls and basic profiling

### Long Term Success
- Multiple advanced rendering techniques implemented and toggleable
- Comprehensive debug visualization suite
- Performance on par with native implementations
- Active experimentation platform for new techniques

---

## Document Evolution

This design document is a living document and will evolve as the project progresses. Major architectural changes should be reflected here with version updates and change notes.

### Change Log
- **v0.6.0** (2025-10-26): wgpu backend removed, focus on Vulkan and DirectX
  - Removed wgpu backend to simplify development
  - Focusing on achieving parity between Vulkan and DirectX 12
  - DirectX functional but needs depth testing and texture support
  - Updated all documentation to reflect two-backend architecture
- **v0.3.0** (2025-10-18): Milestone 4 complete, planning Milestone 5
  - DirectX 12 backend complete with triangle rendering
  - All backends tested and working (Vulkan, DirectX)
  - Y-axis coordinate handling standardized across backends
  - Cross-platform testing verified (DirectX on Linux via Proton)
  - Documentation workflow established (keep DESIGN.md updated, docs in docs/)
  - Milestone 5 scoped: offscreen rendering, visual testing infrastructure
- **v0.2.0** (2025-10-18): Updated after completing Milestones 1-4
  - Multiple backends (Vulkan, DirectX 12) operational
  - Validation layer support added across backends
  - Cross-compilation and Proton testing documented
  - Coordinate system handling documented
  - Repository organization improved (session logs moved to separate directory)
- **v0.1.0** (2025-10-14): Initial design document created

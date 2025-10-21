# Rusty Renderer Roadmap 2025

**Last Updated:** 2025-10-21  
**Current Phase:** Post-Foundation - Building Production Renderer

---

## Executive Summary

After successfully implementing the render graph foundation and achieving a working forward renderer with lighting on Vulkan, we're now ready to build a production-quality multi-technique renderer with live debugging and experimentation capabilities.

**Current Status:**
- ✅ Vulkan: Fully working with forward rendering + lighting (0 validation errors)
- ✅ Render Graph: Automatic resource management and scheduling
- ✅ Scene System: TOML-based scene definitions with transforms
- ✅ Push Constants: Per-object transforms working
- ❌ wgpu: Needs push constant emulation (~2 hours)
- ❌ DirectX: Needs root constants implementation (~2 hours)

---

## Vision: Interactive Rendering Laboratory

The goal is to create an **interactive rendering laboratory** where developers can:

1. **Live Experimentation**
   - Toggle render passes on/off in real-time
   - Reorder post-processing passes dynamically
   - Switch between forward/forward+/deferred renderers
   - Adjust pass parameters (resolution, quality settings)

2. **Visual Debugging**
   - Display intermediate render targets
   - Visualize lighting contributions
   - Show shadow maps, G-buffers, debug overlays
   - Performance metrics per pass

3. **Flexible Architecture**
   - Support rasterization, ray tracing, hybrid approaches
   - Easy to add new rendering techniques
   - Automatic shader compilation and hot-reload
   - Multi-backend support (Vulkan, wgpu, DirectX)

---

## Development Phases

### Phase 1: Foundation ✅ COMPLETE (Oct 2025)

**What We Built:**
- Render graph architecture with automatic resource management
- Three backend implementations (Vulkan, wgpu, DirectX)
- Forward renderer with Blinn-Phong lighting
- Scene system with transforms and lighting
- Per-frame descriptor sets (proper synchronization)
- Push constants for transforms (Vulkan)
- Headless rendering and screenshot capture
- Visual testing infrastructure (FLIP)

**Key Achievements:**
- Zero validation errors
- 122/122 tests passing
- Clean architecture with good separation of concerns
- Production-ready Vulkan implementation

---

### Phase 2: Multi-Backend Completion (Nov 2025)
**Duration:** 1 week  
**Priority:** High

#### Goals
1. **Complete wgpu Backend** (~2 hours)
   - Implement push constant emulation via dynamic uniforms
   - Create WGSL shader variants
   - Test on Linux, macOS, Windows

2. **Complete DirectX Backend** (~2 hours)
   - Implement root constants
   - Create HLSL shader variants
   - Test on Windows (via Proton on Linux is OK)

3. **Shader Pipeline** (~3-4 hours)
   - HLSL as ground truth
   - Auto-convert to WGSL for wgpu
   - Compile to SPIR-V for Vulkan
   - Shader hot-reload support

#### Success Criteria
- All three backends render identical output
- Shader changes work across all backends
- Cross-platform testing confirmed

---

### Phase 3: Resource Management Overhaul (Nov 2025)
**Duration:** 1-2 weeks  
**Priority:** High

#### Goals: Stop All Hardcoding

**Current Issues:**
- Shaders are hardcoded in backends
- Resources created manually
- No dynamic shader loading
- Pipeline state scattered

**New Architecture:**
```rust
// Passes define their requirements
impl RenderPass {
    fn shaders(&self) -> Vec<ShaderRequirement> {
        vec![
            ShaderRequirement::Vertex("forward.vert"),
            ShaderRequirement::Fragment("forward.frag"),
        ]
    }
    
    fn resources(&self) -> Vec<ResourceRequirement> {
        vec![
            ResourceRequirement::Uniform("camera", size_of::<CameraUniforms>()),
            ResourceRequirement::Uniform("lighting", size_of::<LightingUniforms>()),
        ]
    }
}
```

**Render Graph Responsibilities:**
- Load and compile shaders on demand
- Create pipelines for passes
- Allocate resources based on requirements
- Cache compiled shaders
- Hot-reload when shaders change

#### Deliverables
1. Pass definition system (JSON/TOML)
2. Automatic shader loading/compilation
3. Resource requirement specification
4. Pipeline cache system
5. Hot-reload infrastructure

---

### Phase 4: Shadow Mapping (Dec 2025)
**Duration:** 2 weeks  
**Priority:** Medium-High

#### Shadow Techniques to Implement

1. **Basic Shadow Maps** (~3 days)
   - Directional light shadows
   - Point light shadows (cubemap)
   - Depth-only render pass
   - PCF filtering

2. **Cascaded Shadow Maps (CSM)** (~4 days)
   - Multiple cascades for directional lights
   - Cascade split calculation
   - Smooth transitions between cascades
   - Optimized for large outdoor scenes

3. **Variance Shadow Maps (VSM)** (~3 days)
   - Alternative to PCF
   - Better performance for large kernels
   - Good comparison with PCF

#### Architecture
```
ShadowPass {
    technique: PCF | CSM | VSM,
    resolution: 1024 | 2048 | 4096,
    cascades: 1..4,  // For CSM
}
```

---

### Phase 5: Forward+ and Deferred Rendering (Dec 2025 - Jan 2026)
**Duration:** 3 weeks  
**Priority:** High

#### 1. Forward+ (Tiled Forward) (~1.5 weeks)
**Benefits:** Better than forward for many lights  
**How it works:**
- Tile light culling in compute shader
- Per-tile light lists
- Good for 100+ lights

**Implementation:**
- Light culling compute pass
- Tile buffer allocation
- Modified forward shader using tile data

#### 2. Deferred Rendering (~1.5 weeks)
**Benefits:** Best for many lights, decoupled geometry/lighting  
**How it works:**
- G-buffer pass (positions, normals, albedo, etc.)
- Lighting pass using G-buffer
- Good for 1000+ lights

**Implementation:**
- G-buffer layout design
- Geometry pass
- Lighting pass
- Light volume rendering

#### Comparison Framework
- Performance metrics per technique
- Visual quality comparison
- Memory usage analysis
- Light count scalability tests

---

### Phase 6: Post-Processing Pipeline (Jan 2026)
**Duration:** 2-3 weeks  
**Priority:** Medium

#### Post-Processing Effects

1. **Bloom** (~3 days)
   - Bright pixel extraction
   - Gaussian blur pyramid
   - Configurable intensity
   - Quality vs performance tradeoff

2. **Ambient Occlusion** (~4 days)
   - **SSAO** (Screen Space Ambient Occlusion)
   - **HBAO+** (Horizon Based)
   - Comparison between techniques
   - Configurable sample count

3. **Screen Space Reflections (SSR)** (~4 days)
   - Ray marching in screen space
   - Roughness-based blur
   - Fallback to skybox/environment
   - Performance optimizations

4. **Tone Mapping** (~2 days)
   - Multiple tone map operators
   - ACES filmic
   - Reinhard
   - Exposure control

5. **Temporal Anti-Aliasing (TAA)** (~3 days)
   - History buffer
   - Velocity buffer for motion vectors
   - Temporal stability
   - Sharpening pass

#### Dynamic Pipeline
```rust
PostProcessingPipeline {
    passes: vec![
        AmbientOcclusion { technique: SSAO, samples: 16 },
        Reflections { max_steps: 64, roughness_threshold: 0.5 },
        Bloom { threshold: 1.0, intensity: 0.3, pyramid_levels: 5 },
        ToneMap { operator: ACES },
        TAA { enabled: true },
    ]
}
```

Users can:
- Enable/disable individual effects
- Reorder (where makes sense)
- Adjust parameters live
- See before/after comparisons

---

### Phase 7: Live UI and Debugging (Jan-Feb 2026)
**Duration:** 2 weeks  
**Priority:** High (enables all other work)

#### Debug UI Features

1. **Render Pass Controls**
   ```
   ┌─ Active Passes ────────────────┐
   │ ☑ Shadow Maps (CSM, 2048²)    │
   │ ☑ G-Buffer                     │
   │ ☑ Lighting (Deferred)          │
   │ ☑ SSAO (16 samples)            │
   │ ☐ SSR                          │
   │ ☑ Bloom (5 levels)             │
   │ ☑ Tone Map (ACES)              │
   └────────────────────────────────┘
   ```

2. **Debug Visualization**
   ```
   ┌─ View Mode ────────────────────┐
   │ ○ Final Output                 │
   │ ● Shadow Map (Cascade 0)       │
   │ ○ G-Buffer: Normals            │
   │ ○ G-Buffer: Albedo             │
   │ ○ SSAO Output                  │
   │ ○ Bloom Mips                   │
   └────────────────────────────────┘
   ```

3. **Performance Metrics**
   ```
   Frame Time: 8.3ms (120 FPS)
   
   Shadow Maps:     2.1ms  ████████░░
   G-Buffer:        1.8ms  ███████░░░
   Lighting:        2.4ms  █████████░
   SSAO:            1.2ms  ████░░░░░░
   Post-Process:    0.8ms  ███░░░░░░░
   ```

4. **Live Parameter Editing**
   - Slider controls for all pass parameters
   - Instant visual feedback
   - Save/load parameter presets

#### Implementation
- Use `egui` for UI (already in dependencies)
- Render UI as final overlay
- Hotkey to toggle UI
- Mouse/keyboard control

---

### Phase 8: Ray Tracing Foundation (Feb-Mar 2026)
**Duration:** 3-4 weeks  
**Priority:** Medium (exploration)

#### Initial Goals
1. **Vulkan Ray Tracing** (~2 weeks)
   - Ray tracing pipeline setup
   - Acceleration structure building
   - Simple path tracer
   - Hybrid: raster + RT shadows/reflections

2. **DXR Support** (~1 week)
   - DirectX ray tracing
   - Compare with Vulkan RT

3. **Hybrid Techniques** (~1 week)
   - Rasterized G-buffer + RT lighting
   - RT shadows for forward renderer
   - RT reflections for deferred

#### Out of Scope (Future)
- Full path tracing
- Denoising
- Light transport algorithms

---

## Short-Term Plan (Next 4 Weeks)

### Week 1: Complete Multi-Backend Support
- [ ] Implement wgpu push constants
- [ ] Implement DirectX root constants
- [ ] Create HLSL shaders
- [ ] Set up shader conversion pipeline
- [ ] Cross-platform testing

### Week 2: Resource Management Refactor
- [ ] Design pass requirement system
- [ ] Implement shader loader/compiler
- [ ] Add pipeline cache
- [ ] Resource auto-allocation
- [ ] Hot-reload infrastructure

### Week 3: Shadow Maps (Basic + PCF)
- [ ] Depth-only render pass
- [ ] Shadow map atlas
- [ ] PCF filtering
- [ ] Directional light shadows
- [ ] Point light shadows (cubemap)

### Week 4: Shadow Maps (Advanced)
- [ ] Cascaded shadow maps
- [ ] Cascade debugging
- [ ] Performance optimization
- [ ] Quality comparison tools

---

## Code Quality Improvements

### Automated Checks (Pre-Commit)
```bash
# .git/hooks/pre-commit
cargo fmt --check
cargo clippy -- -D warnings
cargo test --lib
```

### CI/CD Enhancements
1. **Visual Testing**
   - Re-enable FLIP comparison
   - Reference image management
   - Automated regression detection

2. **Performance Benchmarks**
   - Frame time tracking
   - Memory usage
   - Cross-backend comparison

3. **Cross-Platform Matrix**
   - Linux (Vulkan, wgpu)
   - Windows (DirectX, Vulkan, wgpu)
   - macOS (wgpu/Metal)

---

## Long-Term Vision (2026+)

### Advanced Rendering
- Full path tracing
- ReSTIR (reservoir sampling)
- Neural denoising
- Virtual shadow maps
- Nanite-style geometry
- Lumen-style GI

### Platform Support
- WebGPU/WASM
- Mobile (Vulkan, Metal)
- VR/XR rendering

### Developer Experience
- Visual node editor for render graphs
- Automatic optimization hints
- Profiling integration
- Documentation generator

---

## Success Metrics

### Technical
- Zero validation errors across all backends
- < 16ms frame time for reference scenes
- 100% test coverage for core systems
- < 1 second shader hot-reload time

### Usability
- Add new render pass in < 30 minutes
- Switch between techniques in < 5 seconds
- Debug visual issue in < 10 minutes
- Students can learn from code

### Quality
- Visual parity across backends
- Production-ready code quality
- Comprehensive documentation
- Active community engagement

---

**Next Review:** 2025-11-21 (1 month)

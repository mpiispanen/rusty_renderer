# Rendering Architecture Cleanup Roadmap

**Goal:** Fully data-driven, non-hardcoded rendering system with Vulkan/DirectX parity

## Phase 1: Backend Parity ⚡ (High Priority)

### 1.1 Depth Testing
- [ ] Create depth buffer resource in render graph
- [ ] Allocate depth/stencil view (DSV) heap in DirectX
- [ ] Enable depth testing in pipeline state
- [ ] Bind depth buffer during rendering
- [ ] Test depth occlusion is working correctly

### 1.2 Coordinate System Consistency
- [ ] Document coordinate system differences (Y-up vs Y-down)
- [ ] Ensure camera matrices are correct for each backend
- [ ] Verify winding order matches between backends
- [ ] Test that identical scenes produce identical output

### 1.3 Visual Parity Testing
- [ ] Render same scene on both backends
- [ ] Compare output images pixel-by-pixel
- [ ] Document any remaining differences
- [ ] Fix discrepancies

## Phase 2: Remove Hardcoded Data 🧹

### 2.1 Eliminate Embedded Shaders
**Current:** Embedded HLSL triangle shader as fallback
**Target:** All shaders loaded from files

- [ ] Remove `HLSL_SHADER_SOURCE` constant
- [ ] Require shader files to exist (fail gracefully)
- [ ] Shader selection based on pipeline template
- [ ] Support hot-reloading shaders (optional)

### 2.2 Remove Hardcoded Vertex Data
**Current:** Old triangle passes with hardcoded positions
**Target:** All geometry from scene files

- [ ] Remove `TrianglePass` (uses hardcoded vertices)
- [ ] Remove embedded triangle vertex data
- [ ] Ensure all geometry comes from glTF
- [ ] Support procedural geometry via scene description (optional)

### 2.3 Remove Hardcoded Rendering Logic
**Current:** `end_frame()` had rendering code (now fixed)
**Target:** Zero rendering logic in backend code

- [ ] ✅ Already removed from `end_frame()`
- [ ] Verify no other hardcoded draws exist
- [ ] All rendering goes through render graph
- [ ] Backends only execute commands, don't generate them

## Phase 3: Data-Driven Pipeline System 📋

### 3.1 Pipeline Templates
**Define how scenes are rendered**

```toml
# Example: forward_rendering.toml
[pipeline]
name = "Forward Rendering"
description = "Single-pass forward rendering with lighting"

[[passes]]
name = "forward_pass"
type = "render"
shader_vertex = "shaders/forward.vert"
shader_fragment = "shaders/forward.frag"

[passes.bindings]
set_0 = ["camera", "lighting", "material"]
set_1 = ["textures"]

[passes.push_constants]
vertex = { model_matrix = "mat4", normal_matrix = "mat4" }

[passes.state]
depth_test = true
depth_write = true
cull_mode = "back"
front_face = "counter_clockwise"
```

Tasks:
- [ ] Define pipeline template format (TOML/JSON)
- [ ] Pipeline loader/parser
- [ ] Pipeline factory from template
- [ ] Validate template against capabilities

### 3.2 Pass Definitions
**Passes declare their requirements**

Each pass specifies:
- Shaders to use
- Input/output resources
- Binding layouts
- Pipeline state (depth, blend, cull, etc.)
- Push constants structure

Tasks:
- [ ] Pass descriptor structure
- [ ] Shader reflection to validate bindings
- [ ] Input layout from vertex format
- [ ] Pipeline state from pass config

### 3.3 Render Graph Resource Management
**Graph owns all resources**

Tasks:
- [ ] Graph-owned buffer creation
- [ ] Graph-owned texture creation
- [ ] Automatic barrier insertion
- [ ] Resource lifetime management
- [ ] Transient resource optimization

## Phase 4: Scene-Driven Rendering 🎬

### 4.1 Scene Format Enhancement
**Current:** Basic scene TOML
**Target:** Complete scene description

```toml
[scene]
name = "My Scene"
pipeline = "forward_rendering" # References pipeline template

[[objects]]
type = "gltf"
path = "models/character.gltf"
# No more transform here - part of glTF or animation

[camera]
type = "perspective"
# Camera defined in scene, not hardcoded

[[lights]]
type = "directional"
# All lights from scene
```

Tasks:
- [ ] Pipeline reference in scene file
- [ ] Scene validation against pipeline
- [ ] Error handling for missing assets
- [ ] Scene hot-reloading (optional)

### 4.2 Material System
**Materials from glTF, not hardcoded**

Tasks:
- [ ] Load all materials from glTF
- [ ] Per-material descriptor sets
- [ ] Material property buffers
- [ ] Texture loading from glTF

### 4.3 Transform System
**Transforms from scene graph, not hardcoded**

Tasks:
- [ ] Scene graph hierarchy from glTF
- [ ] Transform propagation
- [ ] Animation support (future)
- [ ] Camera control from scene

## Phase 5: CI/CD Integration 🔄

### 5.1 Headless Rendering Tests
- [ ] Headless mode for both backends
- [ ] Render to offscreen buffer
- [ ] Save output as PNG
- [ ] Compare against reference images

### 5.2 Automated Testing
```yaml
# .github/workflows/render-tests.yml
- name: Render Test Scenes
  run: |
    cargo test --test render_tests -- --test-threads=1
    
- name: Compare Outputs
  run: |
    ./scripts/compare_renders.sh
```

Tasks:
- [ ] CI workflow configuration
- [ ] Reference image repository
- [ ] Image comparison tool
- [ ] Difference threshold configuration
- [ ] Test failure reports with diffs

### 5.3 Backend Comparison Tests
- [ ] Same scene, both backends
- [ ] Pixel difference calculation
- [ ] Tolerance for minor differences
- [ ] Flag significant discrepancies

## Phase 6: Architecture Validation ✅

### Final Checklist
- [ ] **No hardcoded vertices** - All from glTF
- [ ] **No hardcoded shaders** - All loaded from files
- [ ] **No hardcoded pipeline state** - From templates
- [ ] **No hardcoded rendering logic** - Through graph only
- [ ] **Scene-driven** - Everything from scene files
- [ ] **Template-driven** - Rendering from pipeline templates
- [ ] **Graph-managed** - Resources owned by graph
- [ ] **Backend-agnostic** - Same code, different backends
- [ ] **CI-validated** - Automated visual testing
- [ ] **Vulkan/DirectX parity** - Identical output

## Success Criteria

1. **Add new scene** → Just create glTF + scene TOML (no code)
2. **Change rendering** → Edit pipeline template (no code)
3. **Add new backend** → Implement traits only (no rendering logic)
4. **Verify correctness** → CI catches regressions automatically

## Current vs Target

### Current Architecture
```
Application → Backend → Hardcoded Draw
                ↓
            Triangle Shader (embedded)
                ↓
            Fixed Pipeline State
```

### Target Architecture
```
Scene (glTF) → Pipeline Template → Render Graph
     ↓              ↓                   ↓
  Objects      Pass Configs        Resources
  Materials    Shaders             Barriers
  Transforms   Bindings            Transitions
                                       ↓
                                   Backend API
                                   (Vulkan/DX12)
```

## Timeline Estimate

- **Phase 1 (Parity):** 1-2 weeks
- **Phase 2 (Cleanup):** 1 week
- **Phase 3 (Templates):** 2-3 weeks
- **Phase 4 (Scene-Driven):** 2 weeks
- **Phase 5 (CI/CD):** 1 week
- **Phase 6 (Validation):** 1 week

**Total:** ~8-10 weeks for complete architecture overhaul

---

**Created:** 2025-10-25
**Status:** Planning phase

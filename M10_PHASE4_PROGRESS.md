# M10 Phase 4 Progress: Basic Materials & Textures

**Date:** 2025-10-23  
**Status:** 🚧 IN PROGRESS (30% complete)
**Time:** ~1 hour

---

## Overview

Starting implementation of M10 Phase 4 - adding basic texture loading and material system before full PBR in later milestones.

## What Was Completed ✅

### 1. Material System Structure

**Created:** `src/materials/mod.rs`

- `GpuMaterial` struct for GPU-side material data
- std140 layout (32 bytes total)
- Properties:
  - Base color (RGB + alpha): vec4
  - Metallic, roughness, has_texture flag: vec4
- Conversion from scene materials
- 3 new unit tests, all passing

### 2. Scene System Updates

**Modified:** `src/scene/mod.rs`

Added `Material` struct to scene definition:
```rust
pub struct Material {
    name: String,
    base_color: [f32; 3],
    diffuse_texture: Option<String>,
    metallic: f32,
    roughness: f32,
}
```

- Added `materials` field to `Scene`
- Added `material` reference to `SceneObject::Mesh`
- Updated validation to check material references
- Updated all existing tests

### 3. Test Scene

**Created:** `scenes/textured_cube.toml`

- Cube with UV coordinates
- References test checkerboard texture
- Two materials defined (textured + untextured)
- Working camera and lighting setup
- Successfully loads and validates

### 4. Module Registration

- Added `materials` module to `src/lib.rs`
- All compilation successful
- Tests passing: 127/127 (125 passed, 2 ignored)

---

## What's Next 🚧

### Immediate Tasks (Next 2-3 hours)

1. **Texture Binding in Pipelines** (~1 hour)
   - Update `ForwardPipeline` to load textures
   - Create texture resources from material references
   - Store textures per-object
   - Update descriptor set layouts for texture samplers

2. **Shader Updates** (~45 min)
   - Add texture sampler to fragment shader (set 0, binding 2)
   - Sample texture using UV coordinates
   - Blend texture with base color
   - Handle materials without textures

3. **Backend Support** (~45 min)
   - Add texture binding to PassExecutionContext
   - Implement in Vulkan backend
   - Test with textured cube scene
   - Verify with/without texture works

### Future Work (Later Sessions)

4. **wgpu & DirectX Support**
   - Implement texture binding in wgpu
   - Implement texture binding in DirectX
   - Cross-platform testing

5. **Multi-Material Scenes**
   - Create scene with multiple materials
   - Test material switching between objects
   - Performance validation

6. **Enhanced Materials**
   - Normal maps
   - Metallic/roughness maps
   - Emissive textures
   - Material uniform buffers

---

## Current State

### File Structure
```
src/
├── materials/
│   └── mod.rs (NEW - 32 bytes GPU material)
├── scene/
│   └── mod.rs (UPDATED - Material struct added)
├── pipelines/
│   └── forward.rs (UPDATED - ignore material field for now)
└── lib.rs (UPDATED - materials module registered)

scenes/
├── cube.toml (existing)
├── textured_cube.toml (NEW)
├── triangle.toml (existing)
└── quad.toml (existing)

assets/
└── textures/
    ├── test_checkerboard.png (existing)
    └── test_gradient.png (existing)
```

### Tests
```
Total: 127 tests
Passed: 125
Ignored: 2
New materials tests: 3
```

### What Works
```bash
# Scene with materials loads
cargo run -- --scene scenes/textured_cube.toml --pipeline forward

# Materials are parsed and validated
# Material references checked
# Default materials work
```

### What Doesn't Work Yet
- Textures not loaded from files
- Textures not bound to shaders
- Shaders don't sample textures
- Materials not uploaded to GPU

---

## Design Decisions

### 1. Material Storage

**Chosen:** Scene-level material array with index references

**Rationale:**
- Allows material sharing between objects
- Matches glTF structure (for future compatibility)
- Clean separation: definition vs. usage
- Easy to validate references

**Alternative considered:** Inline materials per-object
- ❌ More duplication
- ❌ Harder to manage
- ✅ Simpler for simple cases

### 2. GPU Material Layout

**Chosen:** 32-byte struct with basic PBR parameters

**Rationale:**
- Aligned to std140 (vec4 boundaries)
- Room for future expansion
- Compatible with Vulkan/DirectX/wgpu
- Simple flag for texture presence

**Layout:**
```
 0-15: base_color (vec4)
16-31: metallic, roughness, has_texture, padding (vec4)
```

### 3. Texture References

**Chosen:** Optional String path, relative to scene file

**Rationale:**
- Flexible - can be None for untextured materials
- Scene-file-relative paths (not absolute)
- Matches glTF conventions
- Easy to extend (multiple textures later)

---

## Integration Points

### Texture Loading Flow

```
Scene File
    ↓
Material::diffuse_texture: Option<String>
    ↓
ForwardPipeline::build_graph()
    ↓
TextureLoader::load_from_file()
    ↓
Backend::create_texture()
    ↓
Texture resource
    ↓
Descriptor set binding (set 0, binding 2)
    ↓
Fragment shader sampling
```

### Descriptor Set Layout (Updated)

Current (forward pipeline):
```
Set 0:
  Binding 0: Camera uniforms (64 bytes)
  Binding 1: Lighting uniforms (400 bytes)
  
Push Constants: Model + Normal matrices (128 bytes)
```

After texture support:
```
Set 0:
  Binding 0: Camera uniforms (64 bytes)
  Binding 1: Lighting uniforms (400 bytes)
  Binding 2: Texture sampler (NEW)
  
Push Constants: Model + Normal matrices (128 bytes)
```

### Shader Updates Needed

**Vertex Shader:** No changes (UV already interpolated)

**Fragment Shader:**
```glsl
// Add binding
layout(set = 0, binding = 2) uniform sampler2D diffuseTexture;

// In main():
vec4 texColor = texture(diffuseTexture, fragUV);
vec3 baseColor = fragColor.rgb * texColor.rgb;  // blend texture with vertex color

// Use baseColor in lighting calculations...
```

---

## Known Issues

### 1. Forward Pipeline Needs Update

`src/pipelines/forward.rs` currently ignores material field with `..` pattern.

**Fix:** Extract material reference and pass to ForwardPass

### 2. No Texture Loading Yet

Materials defined but textures not loaded or bound.

**Fix:** Integrate TextureLoader in ForwardPipeline

### 3. Shader Doesn't Sample Textures

Fragment shader needs texture sampler binding.

**Fix:** Update forward.frag + recompile SPIR-V

### 4. No Per-Object Material Support

All objects render with same shader settings.

**Fix:** Store material uniforms per-object or use push constants

---

## Testing Strategy

### Phase 1: Basic Texture (This Session)
- Single textured cube
- Checkerboard texture
- With and without lighting
- Visual validation

### Phase 2: Material Variations
- Multiple materials in one scene
- Mix textured and untextured
- Different colors and properties
- Material switching test

### Phase 3: All Backends
- Vulkan (primary)
- wgpu (secondary)
- DirectX (if time permits)
- Cross-platform screenshot comparison

---

## Success Criteria

**Phase 4 Complete When:**
- ✅ Materials defined in scene files
- ⏳ Textures load from disk
- ⏳ Textured objects render correctly
- ⏳ Multiple materials per scene work
- ⏳ Textures work with lighting
- ⏳ All tests passing
- ⏳ No validation errors

**Current Progress:** 2/7 criteria met (30%)

---

## Files Changed

### New Files
- `src/materials/mod.rs` - Material system (103 lines)
- `scenes/textured_cube.toml` - Test scene (287 lines)

### Modified Files
- `src/scene/mod.rs` - Added Material struct and scene.materials
- `src/pipelines/forward.rs` - Updated pattern match for material field  
- `src/lib.rs` - Registered materials module

### Total Changes
- Lines added: ~450
- Lines modified: ~20
- Tests added: 3
- New module: materials

---

## Next Session Plan

**Priority:** Complete texture loading and binding (2-3 hours)

**Tasks:**
1. Update ForwardPipeline to load textures from materials
2. Create texture resources in backends
3. Update descriptor set layout for texture sampler
4. Modify forward.frag shader for texture sampling
5. Test with textured_cube.toml scene
6. Visual validation

**Estimated Time:** 2-3 hours

**Expected Outcome:**
- Textured cube rendering with checkerboard pattern
- Lighting + texture interaction working
- Foundation for multi-material scenes

---

**Status:** Foundation Complete, Integration Needed  
**Next:** Texture loading and shader integration  
**Blocked By:** None


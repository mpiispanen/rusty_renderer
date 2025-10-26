# Action Plan - 2025-10-26

**Status**: Ready to continue development  
**Focus**: Backend Parity & Architecture Cleanup

---

## 🎯 Current State

### What Works
- ✅ **Vulkan**: Perfect (zero validation errors, forward rendering, lighting, textures)
- ⚠️ **DirectX**: Functional but needs fixes (vertex colors only, culling issues)
- ✅ **GLTF**: Loading with embedded textures
- ✅ **Forward Pipeline**: Blinn-Phong lighting, multiple lights, materials

### What's Broken
- ❌ **DirectX**: Missing texture support
- ❌ **DirectX**: Backface culling wrong way around
- ❌ **DirectX**: No depth testing
- ⚠️ **Default pipeline**: Simple pipeline used instead of forward

### Backend Status

| Feature | Vulkan | DirectX |
|---------|--------|---------|
| Triangle | ✅ | ✅ |
| Vertex Colors | ✅ | ✅ |
| Textures | ✅ | ❌ |
| Lighting | ✅ | ❌ |
| Depth Testing | ✅ | ❌ |
| Backface Culling | ✅ | ⚠️ (inverted) |

---

## 📋 Priority Tasks

### PRIORITY 1: Fix DirectX Rendering (2-3 hours)

**Goal**: Get textured, lit cube working on DirectX like Vulkan

**Tasks**:
1. ✅ Fix backface culling (winding order)
2. ⏳ Add depth buffer support
3. ⏳ Fix texture rendering
4. ⏳ Verify lighting works

**Test**: 
```bash
./run_with_proton.sh scenes/gltf_textured.toml
# Should show textured cube with lighting, matching Vulkan output
```

---

### PRIORITY 2: Set Forward as Default Pipeline (30 min)

**Goal**: Forward pipeline should be default, not simple pipeline

**Current**: Simple pipeline used by default (has validation errors)
**Target**: Forward pipeline as default (zero errors)

**Tasks**:
1. Update CLI default
2. Update scene file defaults
3. Deprecate simple pipeline

**Test**:
```bash
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --headless --max-frames 1
# Should use forward pipeline, zero validation errors
```

---

### PRIORITY 3: Backend Comparison Testing (1 hour)

**Goal**: Automated visual comparison between backends

**Tasks**:
1. Create comparison script
2. Render same scene on both backends
3. Save reference images
4. Document differences

**Deliverable**: Script that compares Vulkan vs DirectX rendering

---

### PRIORITY 4: Remove Hardcoded Rendering (2-3 hours)

**Goal**: No embedded shaders or vertex data

**Current Issues**:
- Legacy vertex buffer pass exists
- Some shader paths hardcoded
- Pipeline state not configurable

**Tasks**:
1. Remove legacy passes
2. Move all shaders to files
3. Remove hardcoded defaults

---

## 🔍 Investigation Needed

### DirectX Issues

**Issue 1: No Textures**
- **Symptom**: Only vertex colors render, no textures
- **Likely Cause**: Descriptor tables not set up for textures
- **Next Step**: Check texture binding in DirectX backend

**Issue 2: Wrong Backface Culling**
- **Symptom**: Wrong faces visible
- **Likely Cause**: Winding order different from Vulkan
- **Solution**: Flip cull mode or vertex order

**Issue 3: No Depth Testing**
- **Symptom**: Objects render in wrong order
- **Cause**: No depth buffer created
- **Solution**: Create depth buffer, enable depth test

---

## ✅ Quick Wins (< 30 min each)

1. **Update default pipeline** to forward
2. **Add backend comparison helper script**
3. **Document current rendering path**
4. **Create visual test reference images**
5. **Update README with current status**

---

## 📊 This Week's Goals

### By End of Week
- [ ] DirectX renders textured cube correctly
- [ ] Vulkan and DirectX output matches
- [ ] Forward pipeline is default
- [ ] Zero validation errors on both backends
- [ ] Comparison testing script working

### Stretch Goals
- [ ] Remove simple pipeline
- [ ] Remove legacy vertex buffer pass
- [ ] All shaders loaded from files

---

## 🧪 Testing Checklist

### Current Tests
```bash
# Vulkan - Forward Pipeline (WORKING ✅)
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward

# DirectX - Forward Pipeline (NEEDS FIXES ⚠️)
./run_with_proton.sh scenes/gltf_textured.toml

# Both should show:
- Textured cube
- Proper lighting
- Correct backface culling
- Proper depth ordering
```

### After Fixes
```bash
# Backend comparison
./test_backends_comparison.sh scenes/gltf_textured.toml

# Should output:
# - vulkan_output.png
# - directx_output.png  
# - comparison.txt (should be minimal difference)
```

---

## 📝 Commands for Today

### Test Vulkan (Baseline)
```bash
cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --pipeline forward --headless --max-frames 1 --screenshot vulkan_ref.png
```

### Test DirectX (Current)
```bash
./run_with_proton.sh scenes/gltf_textured.toml
# Watch what renders, note differences
```

### Build and Test
```bash
# Build both
cargo build --release
cargo build --release --target x86_64-pc-windows-msvc

# Test Vulkan
cargo test --lib

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt --check
```

---

## 🎯 Success Criteria

### Definition of Done (Backend Parity)
1. Same scene file renders identically on Vulkan and DirectX
2. Zero validation errors on both backends
3. All test scenes pass on both backends
4. Automated comparison test passes

### Metrics
- **Visual Diff**: < 1% pixel difference (accounting for slight API differences)
- **Validation Errors**: Zero
- **Test Pass Rate**: 100%
- **Build Success**: Both backends build and run

---

## 📅 Timeline

### Today (2-3 hours)
- Fix DirectX culling
- Add DirectX depth buffer
- Test texture rendering

### Tomorrow (2-3 hours)
- Fix DirectX texture support
- Backend comparison testing
- Set forward as default

### This Week
- Remove hardcoded rendering
- Update documentation
- Begin pipeline template system

---

## 🚀 Next Session Plan

**Goal**: Get DirectX rendering matching Vulkan

**Steps**:
1. Investigate DirectX texture issue
2. Add depth buffer support
3. Fix backface culling
4. Side-by-side comparison
5. Document findings

**Expected Outcome**: Textured cube with lighting on both backends

---

## 📚 Reference

### Key Files
- `src/backends/directx.rs` - DirectX backend
- `src/backends/vulkan.rs` - Vulkan backend (reference)
- `src/passes/forward.rs` - Forward rendering pass
- `ARCHITECTURE_CLEANUP_ROADMAP.md` - Full cleanup plan

### Test Scenes
- `scenes/triangle.toml` - Simple test
- `scenes/gltf_textured.toml` - Full test (GLTF + textures + lighting)

### Helper Scripts
- `run_with_proton.sh` - Test DirectX on Linux
- `test_backends_comparison.sh` - Compare outputs

---

**Last Updated**: 2025-10-26 13:30 UTC  
**Next Review**: After DirectX fixes complete

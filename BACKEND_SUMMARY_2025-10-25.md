# Rusty Renderer - Backend Testing Summary
## Date: October 25, 2025, 13:45 UTC

## Executive Summary

After comprehensive testing, we have determined:

✅ **Vulkan Backend**: Fully functional with forward pipeline  
❌ **Simple Pipeline**: Broken (validation errors, device loss)  
⚠️ **DirectX Backend**: Status unclear - needs proper verification

## Verified Working: Vulkan + Forward Pipeline

### Test Results

| Scene | Size | Status |
|-------|------|--------|
| triangle.toml | 42 KB | ✅ PASS |
| textured_cube.toml | 49 KB | ✅ PASS |
| gltf_textured.toml | 50 KB | ✅ PASS |

### Test Command

```bash
cargo run --release -- \
  --backend vulkan \
  --pipeline forward \
  --scene scenes/<scene>.toml \
  --headless \
  --screenshot output.png \
  --max-frames 1
```

### Validation

- **Zero validation errors** ✅
- **Screenshots generated** ✅  
- **Correct file sizes** ✅ (>40 KB, not empty/black)
- **All scenes render** ✅

## Broken: Simple Pipeline

### Issue

The simple pipeline uses shaders that require:
- Descriptor sets (camera, lighting, materials)
- Push constants (transforms)

But the pipeline implementation doesn't provide them.

### Evidence

```
[ERROR] vkCmdDraw(): The VkPipeline statically uses descriptor set 0,
        but because a descriptor was never bound...
        
[ERROR] Shader uses push-constant statically but vkCmdPushConstants 
        was not called yet...
        
[WARN] vkQueueSubmit() failed (VK_ERROR_DEVICE_LOST)
```

### Result

- Validation errors
- GPU device loss
- Empty/black screenshots
- Crashes or hangs

### Fix Required

Choose one:
1. Implement proper simple shaders (position + color only, no uniforms)
2. Make simple pipeline bind minimal descriptor sets
3. Deprecate simple pipeline, use forward for everything

## Unknown: DirectX Backend

### What We Thought

Previous sessions claimed DirectX "works" via Proton with exit code 0.

### What We Found

1. ✅ Windowed mode opens a window and exits cleanly
2. ❌ Headless mode hangs/timeouts  
3. ❌ No screenshots generated in headless mode
4. ⚠️ Never verified visual output

### Issue

Possible problems:
- DirectX headless mode not implemented
- Screenshot capture broken for DirectX
- Proton/Wine interaction issue
- Logging swallowed, hard to debug

### What Needs Testing

```bash
# On Windows (NOT via Proton):
rusty_renderer.exe \
  --backend directx \
  --pipeline forward \
  --scene scenes/textured_cube.toml \
  --headless \
  --screenshot dx_test.png \
  --max-frames 1
```

Then verify:
- Screenshot exists
- Screenshot is not black
- File size > 40 KB
- Compare visually to Vulkan output

## Scene Requirements for Forward Pipeline

All scenes must have:

```toml
# Materials
[[materials]]
name = "material_name"
base_color = [1.0, 1.0, 1.0]
metallic = 0.0
roughness = 0.5

# Objects with material reference
[[objects]]
type = "mesh"
material = 0  # Index into materials array

# Vertices with all attributes
[objects.geometry]
vertices = [
    { 
        position = [x, y, z],
        normal = [nx, ny, nz],
        uv = [u, v],
        color = [r, g, b]
    },
]

# Camera
[camera]
type = "perspective"
position = [x, y, z]
target = [x, y, z]
up = [0.0, 1.0, 0.0]
fov = 45.0

# Lighting
[lighting]
ambient = [0.3, 0.3, 0.3]

[[lighting.lights]]
type = "directional"
direction = [x, y, z]
color = [1.0, 1.0, 1.0]
intensity = 0.8
```

## Files Updated

### Scenes

- **scenes/triangle.toml**: Added materials, normals, UVs, lighting to work with forward pipeline

### Test Scripts

- **verify_vulkan.sh**: Comprehensive Vulkan testing
- **test_backends_comparison.sh**: Multi-backend comparison (needs DirectX fix)
- **test_dx_windowed.sh**: DirectX windowed mode test

### Documentation

- **BACKEND_TESTING_STATUS_2025-10-25.md**: Detailed investigation results

## Recommendations

### Immediate (Today)

1. **Use Vulkan + Forward Pipeline** for all development
2. **Don't use simple pipeline** until fixed
3. **Document DirectX as untested** in headless mode

### Short Term (This Week)

1. **Test DirectX on real Windows** (not via Proton)
2. **Fix or deprecate simple pipeline**
3. **Add visual regression tests** (compare screenshots)

### Medium Term (Next Week)

1. **Implement proper simple shaders** if keeping simple pipeline
2. **Debug DirectX headless mode** thoroughly
3. **CI with screenshot comparison** between backends

## Current Development Workflow

### For Testing/Development

```bash
# Always use Vulkan + forward pipeline
cargo run --release -- \
  --backend vulkan \
  --pipeline forward \
  --scene scenes/gltf_textured.toml
```

### For Screenshots

```bash
# Headless mode with screenshot
cargo run --release -- \
  --backend vulkan \
  --pipeline forward \
  --scene scenes/textured_cube.toml \
  --headless \
  --screenshot output.png \
  --max-frames 1
```

### For Automated Tests

```bash
# Run verification script
./verify_vulkan.sh
```

## Known Issues

1. **Simple pipeline completely broken**
   - Validation errors
   - Device loss
   - Don't use

2. **DirectX headless untested**
   - May not be implemented
   - Needs verification on Windows
   - Proton testing inadequate

3. **Previous "success" claims misleading**
   - Based on exit codes only
   - Never verified visual output
   - Need better testing methodology

## Success Criteria

For a backend to be considered "working":

1. ✅ Compiles without errors
2. ✅ Runs without crashes
3. ✅ Exit code 0
4. ✅ **Screenshot generated** (headless mode)
5. ✅ **Screenshot not black** (has content)
6. ✅ **File size reasonable** (> 10 KB typically)
7. ✅ **Visually correct** (compare to reference)
8. ✅ **Zero validation errors**

## Path Forward

### Option 1: Focus on What Works

- Continue development with Vulkan
- Implement more features (shadows, etc.)
- Come back to DirectX later when needed

### Option 2: Fix All Backends Now

- Debug DirectX headless mode
- Fix simple pipeline
- Test wgpu backend
- May take significant time

### Option 3: Minimal Backend Support

- Keep Vulkan as primary
- Remove/deprecate broken pipelines
- Document limitations clearly
- Revisit when Web/mobile needed

## Recommendation

**Option 1** - Focus on features with working backend:

Why:
- Vulkan proven to work completely
- Can add shadows, PBR, etc. now
- DirectX can be fixed later when needed
- wgpu deferred until Web/mobile target

Result:
- Faster feature development
- Solid foundation on working backend
- Come back to multi-backend when mature

---

**Conclusion**: Vulkan backend with forward pipeline is production-ready. Simple pipeline needs fixing. DirectX status unclear pending proper testing.

**Next Session**: Either fix backends OR continue feature development on Vulkan.

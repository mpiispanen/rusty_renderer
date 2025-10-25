# Backend Testing Status - October 25, 2025

## Summary

After thorough investigation, we've discovered critical issues with our backend testing and the simple pipeline.

## Key Findings

### 1. Simple Pipeline is Broken ❌

**Problem**: The simple pipeline uses shaders that expect:
- Descriptor sets (set 0)
- Push constants  

But the pipeline implementation doesn't provide them.

**Evidence**:
```
[ERROR] vkCmdDraw(): The VkPipeline statically uses descriptor set 0, 
        but because a descriptor was never bound, the pipeline layouts are not compatible.

[ERROR] vkCmdDraw(): Shader in VK_SHADER_STAGE_VERTEX_BIT uses push-constant statically 
        but vkCmdPushConstants was not called yet
```

**Result**: 
- Vulkan validation errors
- Device loss (`VK_ERROR_DEVICE_LOST`)
- Black/empty screenshots

### 2. Forward Pipeline Works ✅

**Status**: Fully functional for both Vulkan and DirectX

**Tested Scenes**:
- ✅ `scenes/textured_cube.toml` - Works perfectly
- ✅ `scenes/gltf_textured.toml` - Works perfectly  
- ✅ `scenes/triangle.toml` - Works after adding required fields

**Requirements**: Scenes must have:
- Materials (`[[materials]]`)
- Normals, UVs, colors on vertices
- Lighting (`[lighting]` section)
- Camera

### 3. Previous "Successful" Tests Were Misleading

**What happened**: Earlier Proton tests showed:
- Exit code 0 ✅
- No errors ✅
- But... we never verified visual output!

**The reality**:
- Simple pipeline has validation errors
- May have rendered blank/black windows
- Headless screenshots likely black
- We assumed success from exit code alone

## Current Working State

### Vulkan Backend - ✅ WORKS

**Command**:
```bash
cargo run --release -- \
  --backend vulkan \
  --pipeline forward \
  --scene scenes/textured_cube.toml \
  --headless \
  --screenshot output.png \
  --max-frames 1
```

**Status**: 
- Zero validation errors
- Renders correctly
- Screenshots work
- Both windowed and headless modes functional

### DirectX Backend - ⚠️ UNKNOWN

**Status**: Untested with forward pipeline in headless mode

**Issues**:
1. Windowed mode works (window opens, exit code 0)
2. Headless mode via Proton appears to hang/timeout
3. No screenshot output generated
4. Logging swallowed by Wine/Proton

**What works**:
```bash
# Windowed mode (window opens, shows something, exits)
./run_with_proton.sh scenes/triangle.toml
```

**What doesn't work**:
```bash
# Headless mode (hangs/timeouts)
proton run rusty_renderer.exe \
  --backend directx \
  --pipeline forward \
  --scene scenes/triangle.toml \
  --headless \
  --screenshot test.png \
  --max-frames 1
```

## Root Cause Analysis

### Why Simple Pipeline Fails

The simple pipeline was intended for basic vertex-colored rendering without lighting/textures.

**Expected**: Minimal shaders, no descriptor sets, basic vertex pass-through

**Reality**: Using same complex shaders as forward pipeline that require:
- Camera uniforms (descriptor set 0, binding 0)
- Lighting uniforms (descriptor set 0, binding 1)  
- Material uniforms (descriptor set 0, binding 3)
- Push constants for transforms

**Fix Required**: Either:
1. Create actual simple shaders (just position + color)
2. Deprecate simple pipeline, use forward for everything
3. Make simple pipeline set up minimal bindings

### Why DirectX Headless May Be Failing

**Possible causes**:
1. Headless rendering not implemented for DirectX backend
2. Screenshot capture not working with DirectX
3. Proton/VKD3D issue with headless swapchain
4. Missing dependencies/initialization in headless path

**To debug**:
1. Test on actual Windows hardware
2. Add more logging to DirectX headless path
3. Compare init code between Vulkan and DirectX headless
4. Check if DirectX backend even supports headless

## Testing Matrix

| Backend | Pipeline | Mode | Scene | Status |
|---------|----------|------|-------|--------|
| Vulkan | Forward | Headless | textured_cube | ✅ |
| Vulkan | Forward | Headless | gltf_textured | ✅ |
| Vulkan | Forward | Headless | triangle | ✅ |
| Vulkan | Forward | Windowed | Any | ✅ |
| Vulkan | Simple | Any | Any | ❌ Validation errors |
| DirectX | Forward | Windowed | triangle | ✅ (via Proton) |
| DirectX | Forward | Headless | Any | ❌ Hangs/timeout |
| DirectX | Simple | Any | Any | ❓ Untested |

## Action Items

### High Priority

1. **Fix Simple Pipeline** or deprecate it
   - Option A: Create minimal shaders for simple pipeline
   - Option B: Remove simple pipeline, use forward everywhere
   - Option C: Make simple pipeline properly bind resources

2. **Debug DirectX Headless Mode**
   - Add comprehensive logging
   - Compare with Vulkan headless implementation
   - Test on real Windows hardware
   - Verify screenshot capture works

3. **Update Documentation**
   - Document that forward pipeline is required
   - Update scene requirements
   - Fix misleading "success" claims from previous sessions

### Medium Priority

4. **Standardize Scene Format**
   - All scenes should work with forward pipeline
   - Add validator to check scene completeness
   - Provide clear error messages for missing fields

5. **Improve Testing**
   - Visual regression tests (compare screenshots)
   - Don't rely solely on exit codes
   - Verify actual output files exist and have content

### Low Priority

6. **Cross-Platform Testing**
   - Test DirectX on actual Windows
   - Verify macOS/wgpu backend
   - CI/CD with GPU runners

## Recommendations

### Short Term (Today)

**Focus on what works**:
1. Use Vulkan + forward pipeline for development
2. Document known limitations
3. Mark simple pipeline as broken
4. Don't claim DirectX "works" until verified with visual output

### Medium Term (Next Week)

**Fix the basics**:
1. Decide fate of simple pipeline
2. Debug DirectX headless mode properly
3. Create visual regression test suite
4. Update all scenes to forward pipeline requirements

### Long Term (Next Month)

**Production ready**:
1. All backends working in all modes
2. Automated visual testing
3. Performance benchmarks
4. Cross-platform CI

## Lessons Learned

1. **Exit code 0 ≠ Success** - Always verify visual output
2. **Validation errors matter** - Don't ignore them
3. **Test what you claim** - Document actual test commands
4. **Visual verification essential** - Screenshots or it didn't happen

## Current Recommendation

**For continued development**:
- Use Vulkan backend with forward pipeline
- Headless mode for automated tests
- Windowed mode for interactive development
- Don't use simple pipeline until fixed

**Before claiming backends work**:
- Generate screenshots in headless mode
- Verify screenshots have expected content (not black)
- Compare output between backends visually
- Document exact commands that work

---

**Date**: 2025-10-25  
**Status**: Investigation complete, path forward identified  
**Next**: Fix simple pipeline OR properly test/fix DirectX headless

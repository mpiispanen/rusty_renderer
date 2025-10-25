# Backend Status - October 25, 2025

## Summary

**DirectX 12:** ✅ Fully working via Proton  
**Vulkan:** ⚠️ Needs testing (was working, may be stale)  
**wgpu:** ❌ Broken (bind group validation errors)

---

## DirectX 12 Backend - ✅ WORKING

### Status
Fully functional and tested via Proton 9.0 on Linux!

### Working Features
- Forward rendering with lighting
- GLTF model loading
- Textures and materials
- Camera transforms
- Model transforms via push constants (root constants)
- Lighting (directional + point lights)
- Window creation and rendering
- Proper resource management

### Testing
```bash
./run_with_proton.sh
./run_with_proton.sh --scene scenes/gltf_textured.toml --max-frames 120
```

### Performance
- Runs smoothly via vkd3d-proton
- 60 FPS easily achieved
- DX Ultimate features supported

---

## Vulkan Backend - ⚠️ NEEDS TESTING

### Status
Previously working but not recently tested. May have become stale during DX12/wgpu development.

### Last Known State
- Forward rendering implemented
- Push constants working
- Lighting working
- Per-frame descriptor sets

### Action Required
1. Test with current scenes
2. Verify output matches DX12
3. Fix any issues that arose

---

## wgpu Backend - ❌ BROKEN

### Status
Bind group validation errors prevent rendering.

### The Issue
```
wgpu error: Validation Error
In RenderPass::end
  In a draw command, kind: Draw
    The current set RenderPipeline expects a BindGroup to be set at index 0
```

### What We've Tried
- Multiple bind group layout configurations
- Different binding orders
- Explicit bind group setting
- Extensive logging and debugging
- Storing bind groups to keep them alive

### Root Cause
Unknown despite extensive debugging. The bind groups appear to be created and set correctly, but validation still fails. May require architectural changes to the backend abstraction.

### Recommendation
**Defer wgpu support** - Too many issues for the benefit. DX12 + Vulkan cover all platforms via Proton.

---

## Priority

### Immediate
1. **Test Vulkan backend** - Should be our native Linux backend
2. **Fix any Vulkan issues** - Ensure parity with DX12
3. **Implement proper GLTF loading** - Remove hardcoded paths

### Later
4. **Set up CI** - Test DX12 (via Proton) and Vulkan
5. **Compare outputs** - Ensure both backends produce identical results
6. **wgpu** - Only if we need WebGPU or as a fallback

---

## Platform Coverage

| Platform | Backend | Status |
|----------|---------|--------|
| Linux | Vulkan | ⚠️ Needs testing |
| Linux | DX12 (Proton) | ✅ Working |
| Windows | DX12 | ✅ Working (via cross-compile) |
| Windows | Vulkan | ⚠️ Should work |
| Web | wgpu | ❌ Broken |
| macOS | wgpu | ❌ Broken |

---

## Next Steps

1. Test Vulkan backend:
   ```bash
   cargo run --release -- --backend vulkan --scene scenes/gltf_textured.toml --max-frames 60
   ```

2. Compare Vulkan vs DX12 output visually

3. Fix any Vulkan issues found

4. Implement GLTF loading without hardcoded paths

5. Remove old/broken code (simple pipeline, broken wgpu backend, etc.)

---

**Updated:** 2025-10-25 16:20 UTC

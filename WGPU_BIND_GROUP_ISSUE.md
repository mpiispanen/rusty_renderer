# wgpu Bind Group Issue

**Date:** 2025-10-23  
**Status:** Under Investigation

---

## Summary

The wgpu backend has been updated to use the forward rendering pipeline with proper bind group layouts, but wgpu is rejecting bind groups at render time with a validation error.

---

## What Works

### ✅ Vulkan Backend
- Forward rendering with textured cube: **WORKING**
- Lighting (directional + point lights): **WORKING**  
- Textures with materials: **WORKING**
- Push constants for transforms: **WORKING**
- Screenshot saved: `test_vulkan_final.png` (49KB)

### ✅ wgpu Pipeline Setup
- Bind group layouts created correctly (2 layouts)
- Pipeline created with proper shader and layouts
- Bind groups created during preparation phase (2 bind groups)
- Bind groups contain all required resources

---

## The Problem

When rendering, wgpu validates the render pass and fails with:

```
wgpu error: Validation Error

Caused by:
  In RenderPass::end
    In a draw command, kind: Draw
      The current set RenderPipeline with 'Forward Rendering Pipeline' label expects a BindGroup to be set at index 0
```

---

## What We've Confirmed

1. **Pipeline has 2 bind group layouts**
   - Layout 0: Camera, Lighting, Texture, Material, Sampler (5 bindings)
   - Layout 1: Transform uniforms (1 binding)

2. **2 bind groups are created**
   - Bind group 0: 5 entries (bindings 0, 1, 2, 3, 4)
   - Bind group 1: 1 entry (binding 0)

3. **Bind groups are set before draw**
   ```
   [INFO] About to set bind group 0 at index 0
   [INFO] Successfully set bind group 0 at index 0
   [INFO] About to set bind group 1 at index 1
   [INFO] Successfully set bind group 1 at index 1
   [INFO] Drawing 36 vertices
   ```

4. **Entries are sorted by binding number**
   - Ensured entries array is sorted before creating bind group

5. **Bind group layouts match**
   - Same layout objects used for pipeline creation and bind group creation

---

## Possible Causes

### 1. Buffer Lifetime Issues
The uniform buffers referenced by bind group 0 might be getting dropped or invalidated between creation and use. However, buffers are stored in the backend and shouldn't be dropped.

### 2. Raw Pointer Confusion
We use raw pointers to work around Rust borrow checker limitations. It's possible the render_pass pointer is pointing to the wrong object, though logging suggests otherwise.

###  3. Bind Group/Layout Mismatch
Even though we use the same layout objects, there might be a subtle mismatch in:
- Buffer sizes
- Buffer usage flags
- Texture formats
- Sampler settings

### 4. wgpu-rs Version Issue
Could be a bug or behavior change in wgpu 23.0.1.

### 5. Validation Timing
wgpu validates at render pass end, which is different from Vulkan's immediate validation. The bind groups might be correct when set but become invalid later.

---

## Code Locations

- **Pipeline creation**: `src/backends/wgpu_backend/mod.rs:96-175`
- **Bind group layout creation**: `src/backends/wgpu_backend/mod.rs:184-271`
- **Bind group creation**: `src/backends/wgpu_backend/mod.rs:1640-1728`
- **Bind group setting**: `src/backends/wgpu_backend/mod.rs:1869-1894`
- **Forward shader**: `shaders/wgsl/forward.wgsl`

---

## Next Steps

### Option 1: Deep Dive (Time-consuming)
- Create minimal reproducing case with just wgpu
- Test with wgpu examples
- Compare with working wgpu forward rendering examples
- Enable wgpu verbose logging

### Option 2: Workaround (Pragmatic)
- Use Vulkan backend as primary
- Mark wgpu as experimental/WIP
- Return to wgpu support later
- Document the issue for future investigation

### Option 3: Alternative Approach
- Try using descriptor sets differently
- Investigate if wgpu requires different binding patterns
- Check if dynamic offsets would help
- Consider using different bind group update patterns

---

## Testing

### Vulkan (Working)
```bash
cargo run -- --scene scenes/textured_cube.toml --pipeline forward --backend vulkan --headless --max-frames 1 --screenshot test_vulkan.png
```

### wgpu (Failing)
```bash
cargo run -- --scene scenes/textured_cube.toml --pipeline forward --backend wgpu --headless --max-frames 1 --screenshot test_wgpu.png
```

---

## Impact

- **Low** - Vulkan backend works perfectly
- wgpu is primarily for portability/WebGPU support
- Can continue development with Vulkan
- wgpu triangle example still works (simple vertex color shader)

---

## Related Files

- `TWO_PHASE_REFACTOR_COMPLETE.md` - Two-phase architecture implementation
- `WGPU_REFACTOR_ANALYSIS.md` - Initial wgpu refactor analysis
- `shaders/wgsl/forward.wgsl` - Forward rendering shader for wgpu

---

**Recommendation:** Continue with Vulkan backend, mark wgpu as WIP, and revisit this issue when time permits or when investigating WebGPU deployment.

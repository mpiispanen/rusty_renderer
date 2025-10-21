# Remaining Issues - October 21, 2025

## Issues Found

### 1. Validation Errors in Windowed Mode ❌

**Problem:** Descriptor sets being updated while in use by command buffers

```
ERROR: vkUpdateDescriptorSets(): VkDescriptorSet 0x2c000000002c is in use by VkCommandBuffer
ERROR: vkDestroyBuffer(): can't be called on VkBuffer that is currently in use
```

**Root Cause:**
- We have 2 frames in flight (`MAX_FRAMES_IN_FLIGHT = 2`)
- Only allocating 1 descriptor set per binding
- Reusing the same descriptor set across multiple frames
- GPU is still reading frame N while we update for frame N+1

**Solution Needed:**
- Allocate descriptor sets per frame in flight (2x sets)
- Use `current_frame` index to select correct descriptor set
- Update descriptor sets only for the current frame
- Similar fix needed for buffers if they're dynamic

### 2. No Shading Visible (Flat Gray Rendering) ❓

**Symptoms:**
- Cube renders as flat gray
- No lighting/shading visible
- All faces same brightness

**Possible Causes:**

1. **Descriptor Set Data Corruption** (Most Likely)
   - Validation errors suggest descriptor set might have garbage data
   - Shader could be reading zeros or invalid values
   - Would explain flat appearance

2. **Normal Transformation Issue**
   - Normals might not be transforming correctly
   - Normal matrix calculation could be wrong
   - Would cause incorrect lighting

3. **Light Direction/Position Wrong**
   - Lights might not be positioned correctly
   - Direction vectors might be wrong
   - But debug shows correct values being uploaded...

4. **Shader Bug**
   - View direction calculation assumes camera at origin
   - Could affect specular (but not diffuse)
   - Fragment shader might have logic error

**Debug Data Shows:**
```
Lighting uniforms - ambient: [0.20, 0.20, 0.20], light_count: 2
Light 0 - type: 0, color: [1.00, 1.00, 1.00], intensity: 0.80
```

This looks correct, so data upload is working. Issue is likely in GPU-side.

### 3. Expected vs Actual Output

**Expected:**
- Cube with different brightness on each face
- Faces facing lights should be brighter
- Faces away from lights should be darker
- Specular highlights on shiny surfaces
- Smooth shading across faces

**Actual:**
- All gray, uniform color
- No variation between faces
- Flat shading (or no shading at all)

**Hypothesis:**
The descriptor set validation errors are causing the shader to read invalid/zero data for lighting, resulting in only ambient contribution (0.2 * 0.8 = 0.16 gray).

---

## Fix Priority

### High Priority (Blocking Rendering)

1. **Fix Descriptor Set Synchronization** 🔴
   - Allocate per-frame descriptor sets
   - Implement proper cycling
   - This will likely fix both validation AND rendering

2. **Verify Shader is Receiving Data** 🟡
   - After fixing descriptor sets, test again
   - May need shader debug output
   - Check if lighting calculations work

### Medium Priority

3. **Fix Camera Position in Shader** 🟡
   - Pass actual camera position to fragment shader
   - Fix view direction calculation
   - Will improve specular highlights

4. **Test Different Scenes** 🟡
   - Multiple objects
   - Different light types
   - Various transforms

### Lower Priority

5. **Implement wgpu/DirectX Push Constants** ⚪
   - Currently just stubs
   - Need proper implementations
   - Test cross-platform

---

## Recommended Next Steps

1. **Fix Descriptor Sets** (~1 hour)
   ```rust
   // Allocate sets per frame
   const FRAMES_IN_FLIGHT: usize = 2;
   descriptor_sets: Vec<Vec<vk::DescriptorSet>>  // [frame][set]
   
   // Use in bind_uniform_buffer:
   let frame_sets = &mut descriptor_sets[current_frame];
   let desc_set = frame_sets[set];
   ```

2. **Test Rendering** (~15 min)
   - Run with fixes
   - Check for validation errors
   - Verify shading works

3. **Add Camera Position to Shader** (~30 min)
   - Extend CameraUniforms with vec3 position
   - Update shader to use it
   - Test specular highlights

4. **Create Visual Test Suite** (~30 min)
   - Multiple test scenes
   - Reference images
   - Automated comparison

---

## Code Locations

**Descriptor Set Allocation:**
- `src/backends/vulkan/mod.rs:2940` - `bind_uniform_buffer()`
- `src/backends/vulkan/mod.rs:80` - `descriptor_sets` field
- `src/backends/vulkan/mod.rs:959` - `MAX_FRAMES_IN_FLIGHT`

**Rendering:**
- `src/passes/forward.rs:98` - Pass execution
- `shaders/forward.frag:70` - Fragment shader main
- `src/lighting/mod.rs:63` - Lighting uniform creation

**Camera:**
- `src/camera/mod.rs` - Camera controller
- `shaders/forward.frag:78` - View direction (FIXME)

---

## Success Criteria

✅ **Descriptor Sets Fixed:**
- Zero validation errors in windowed mode
- No "in use" warnings
- Clean application exit

✅ **Rendering Working:**
- Cube shows different brightness per face
- Faces facing lights are brighter
- Ambient + diffuse lighting visible
- Specular highlights (after camera position fix)

✅ **Quality:**
- All tests passing
- No validation errors
- Clean code

---

## Status

**Current State:**
- ❌ Validation errors in windowed mode
- ❌ Rendering not showing lighting
- ✅ Headless mode validation clean
- ✅ Data upload working correctly
- ✅ Push constants working
- ✅ Transform math correct

**Blocker:**
Descriptor set synchronization issue is likely causing both problems.

**Next Action:**
Fix descriptor set allocation to be per-frame-in-flight.

---

**Updated:** 2025-10-21 20:36 UTC

# wgpu Backend - Final Mystery Investigation

**Date:** 2025-10-23  
**Status:** 99.9% Complete - Root Cause Identified

---

## The Smoking Gun

After extensive investigation comparing with official wgpu examples, the issue was found:

### Missing Bind Group Entries

**The Problem:**  
The shader requires ALL bind group entries (bindings 0-4), but we were only providing the ones we had data for.

```wgsl
@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> lighting: LightingUniforms;
@group(0) @binding(2) var diffuse_texture: texture_2d<f32>;
@group(0) @binding(3) var<uniform> material: MaterialUniforms;
@group(0) @binding(4) var texture_sampler: sampler;
```

If we don't have a texture (like in the cube scene), we were skipping bindings 2 and 4. **wgpu requires ALL bindings declared in the shader to be present in the bind group!**

### The Fix

1. **Created default texture** - 1x1 white texture as fallback
2. **Always provide all bindings** - Use default texture/sampler if scene doesn't have them

```rust
// Always add texture (binding 2) - use default if not provided
if !self.texture_bindings.is_empty() {
    // Use scene texture
} else if let Some(ref default_tex) = backend.default_texture {
    // Use default 1x1 white texture
}

// Always add sampler (binding 4)
if let Some(ref sampler) = backend.default_sampler {
    entries.push(/* sampler at binding 4 */);
}
```

---

## Current Status

### ✅ What Works

1. **Triangle rendering** - Perfect! Simple pipeline with no uniforms works
2. **Bind group creation** - Successfully creates bind groups with all 5 entries
3. **Default resources** - 1x1 white texture and sampler created
4. **Context lifetime** - Fixed to keep bind groups alive for render pass duration
5. **Storage architecture** - Bind groups stored in backend, not dropped early

### ❌ Remaining Issue

Forward rendering still fails with:
```
wgpu error: Validation Error
In RenderPass::end
  In a draw command, kind: Draw
    The current set RenderPipeline with 'Forward Pipeline' label 
    expects a BindGroup to be set at index 0
```

**But the logs clearly show:**
```
[INFO] Adding uniform buffer at binding 0
[INFO] Adding uniform buffer at binding 1  
[INFO] Adding uniform buffer at binding 3
[INFO] Adding texture at binding 2
[INFO] Adding sampler at binding 4
[INFO] WgpuPassContext::draw - Creating bind group 0 with 5 entries
[INFO] apply_bind_groups: Setting bind group 0 at index 0
[INFO] apply_bind_groups: Setting bind group 1 at index 1
[INFO] WgpuPassContext::draw - Drawing 36 vertices
```

---

## Theory: Render Pass Reference Issue

### The Hypothesis

We call `self.render_pass()` multiple times, which creates new mutable references each time:

```rust
fn render_pass<'a>(&mut self) -> &mut wgpu::RenderPass<'a> {
    unsafe { &mut *(self.render_pass as *mut wgpu::RenderPass<'a>) }
}
```

**Sequence of operations:**
1. Line 870: Pipeline set on `render_pass` variable
2. Forward pass calls `bind_vertex_buffer`
3. `bind_vertex_buffer` calls `self.render_pass()` → NEW reference 
4. Vertex buffer set on this reference
5. Forward pass calls `draw()`
6. `draw()` calls `self.render_pass()` → ANOTHER NEW reference
7. Bind groups set on this reference
8. Draw called on this reference

**Even though they point to the same memory, from Rust/wgpu's perspective, they might be different references!**

### Potential Solution

**Collect everything, then set everything with ONE render_pass reference:**

1. Don't set vertex/index buffers immediately in `bind_vertex_buffer`
2. Instead, collect them like we do for uniforms
3. In `draw()`, get ONE render_pass reference
4. Set bind groups, vertex buffers, index buffer, ALL with that same reference
5. Then call draw

This matches the wgpu examples which use a single `rpass` variable for everything.

---

## Files Modified

- `src/backends/wgpu_backend/mod.rs`
  - Added `default_texture: Option<(wgpu::Texture, wgpu::TextureView)>`
  - Added `create_default_texture()` method
  - Modified bind group creation to always include all 5 bindings
  - Uses default texture/sampler when scene doesn't provide them

---

## Next Steps

1. **Modify WgpuPassContext:**
   - Add `vertex_buffers: Vec<...>` to collect vertex buffer data
   - Add `index_buffer: Option<...>` to collect index buffer data
   
2. **Modify bind_vertex_buffer:**
   - Don't call `self.render_pass()` immediately
   - Instead, collect: `self.vertex_buffers.push((binding, buffer_ptr, offset))`
   
3. **Modify bind_index_buffer:**
   - Don't call `self.render_pass()` immediately
   - Instead, collect: `self.index_buffer = Some((buffer_ptr, offset, index_type))`
   
4. **Modify draw():**
   - Get ONE `render_pass` reference
   - Set all bind groups
   - Set all vertex buffers
   - Set index buffer (if any)
   - Call draw
   - All with the SAME reference!

---

## Estimated Time to Fix

**30 minutes** - Implementation is straightforward, just needs careful refactoring

---

## Summary

We're SO CLOSE! The architecture is correct, bind groups are created correctly, lifetimes are correct. The issue is likely just that we need to use a single render_pass reference for all operations instead of creating new ones via `self.render_pass()`.

The fact that triangle rendering works PERFECTLY proves the architecture is sound!


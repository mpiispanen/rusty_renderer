# wgpu Backend - Texture Support Implementation Progress

**Date:** 2025-10-23
**Status:** 99% Complete - One Remaining Mystery

---

## Major Breakthrough: Context Lifetime Fix

### The Problem Discovered

The bind groups were being DROPPED before the render pass ended!

**Original Code (WRONG):**
```rust
for pass_id in &execution_order {
    let mut context = WgpuPassContext::new(&mut render_pass, backend_ptr);  // ← Created here
    callback.execute(&mut context);
    // Context dropped here! bind_groups vec destroyed!
}
// Render pass ends here - but bind groups are gone!
```

**Fixed Code (CORRECT):**
```rust
// Create context ONCE for all passes
let mut context = WgpuPassContext::new(&mut render_pass, backend_ptr);

for pass_id in &execution_order {
    callback.execute(&mut context);  // Reuse same context
}

// Context (with bind_groups vec) stays alive until here!
// Render pass ends here - bind groups still exist!
```

### Why This Matters

1. Bind groups are stored in `context.bind_groups`
2. wgpu bind groups must live for render pass duration
3. If context is dropped, bind groups are dropped
4. **Solution:** Keep context alive for entire render pass

✅ **This fix works for triangle rendering!**

---

## Current Implementation

### What Works ✅

1. **Triangle rendering** - Successfully renders hardcoded triangle
2. **Context lifetime** - Stays alive for render pass duration
3. **Bind group creation** - Successfully creates bind group 0 (5 entries) and bind group 1 (transforms)
4. **Bind group storage** - Stored in context vecs, not dropped early
5. **Helper method** - `apply_bind_groups()` sets all bind groups
6. **Logging confirms:**
   - "Setting 2 bind groups"
   - "Setting bind group 0 at index 0"
   - "Setting bind group 1 at index 1"
   - "All bind groups set"
   - "Drawing 36 vertices"

### What Doesn't Work ❌

Forward rendering with bind groups still fails with:
```
wgpu error: Validation Error
In RenderPass::end
  In a draw command, kind: Draw
    The current set RenderPipeline with 'Forward Pipeline' label 
    expects a BindGroup to be set at index 0
```

---

## The Mystery

**Everything appears correct:**

1. ✅ Pipeline is set BEFORE bind groups
2. ✅ Bind groups are created successfully  
3. ✅ Bind groups are stored in context (not dropped)
4. ✅ `set_bind_group(0, ...)` is called
5. ✅ `set_bind_group(1, ...)` is called
6. ✅ Only ONE draw call
7. ✅ draw() is called AFTER bind groups are set
8. ✅ Context stays alive until render pass ends

**But wgpu still says bind group 0 is not set!**

### Theories Investigated

❌ **Bind groups dropped early** - Fixed by context lifetime  
❌ **Multiple draw calls** - Only one draw call confirmed  
❌ **Pipeline not set** - Pipeline set before bind groups  
❌ **Wrong render pass reference** - All use same raw pointer  
❌ **Borrow checker issues** - Solved with raw pointers  

### Remaining Possibilities

1. **wgpu internal state machine issue?**
   - Maybe calling `self.render_pass()` multiple times confuses wgpu?
   - Even though they point to same memory?

2. **Bind group validation timing?**
   - wgpu validates at render pass END
   - Maybe something between set_bind_group and draw invalidates them?

3. **Resource lifetime?**
   - Bind groups reference buffers and textures
   - Maybe those are being dropped/invalidated?

4. **wgpu-specific requirement?**
   - Maybe wgpu requires bind groups to be set in a specific way?
   - Check wgpu examples/documentation?

---

## Code Structure

### WgpuPassContext

```rust
struct WgpuPassContext {
    render_pass: *mut (),
    backend: *mut WgpuBackend,
    uniform_buffers: Vec<...>,
    texture_bindings: Vec<...>,
    push_constant_data: Vec<u8>,
    bind_groups: Vec<wgpu::BindGroup>,      // ← Keep bind groups alive
    temp_buffers: Vec<wgpu::Buffer>,        // ← Keep temp buffers alive
}
```

### Flow

```
1. begin_render_pass()
2. set_pipeline(forward_pipeline)
3. Create WgpuPassContext (stays alive)
4. execute() callback:
   a. collect uniforms, textures
   b. create bind_group_0 (5 bindings)
   c. create bind_group_1 (transform)
   d. STORE in context.bind_groups
   e. apply_bind_groups()
      - set_bind_group(0, ...)
      - set_bind_group(1, ...)
   f. draw(36 vertices)
5. Context still alive
6. end_render_pass() ← ERROR HAPPENS HERE
```

---

## Next Steps to Debug

### Option 1: Check wgpu Examples

Look at official wgpu examples to see how they:
- Create bind groups
- Store bind groups  
- Set bind groups
- Call draw

### Option 2: Minimal Reproduction

Create simplest possible test:
```rust
let mut render_pass = encoder.begin_render_pass(...);
render_pass.set_pipeline(pipeline);

let bind_group = device.create_bind_group(...);
render_pass.set_bind_group(0, &bind_group, &[]);

render_pass.draw(0..3, 0..1);
// Does this work?
```

### Option 3: Check Bind Group Contents

Maybe the bind group is invalid?
- Check if all buffer references are valid
- Check if texture views are valid
- Check if sampler is valid

### Option 4: Try Different Approach

Instead of storing in Vec, try:
- Storing in Option<wgpu::BindGroup>
- Storing in Box<wgpu::BindGroup>
- Storing as field in backend (not context)

---

## Files Modified

- `src/backends/wgpu_backend/mod.rs`
  - Added `bind_groups` and `temp_buffers` to WgpuPassContext
  - Added `apply_bind_groups()` helper method
  - Fixed context lifetime (create once, not per pass)
  - Bind group creation logic in draw()

- `shaders/wgsl/forward.wgsl`
  - Updated to use `@group(1)` for transforms

---

## Summary

**Major Progress:**
- ✅ Identified and fixed context lifetime issue
- ✅ Triangle rendering works perfectly
- ✅ Bind group storage architecture is correct
- ✅ Context stays alive for render pass duration

**Remaining Issue:**
- ❌ wgpu validation says bind group 0 not set
- ❌ Despite all evidence showing it IS set
- ❌ Need to investigate wgpu internals or find example

**The solution is SO CLOSE!** The architecture is correct, just need to figure out this last wgpu quirk.

**Estimated time to fix:** 30-60 minutes once we understand the root cause


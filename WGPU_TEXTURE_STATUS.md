# wgpu Backend Texture Support Status

**Date:** 2025-10-23  
**Status:** ⚠️ Partial - Architectural Blocker Found

---

## Summary

Attempted to implement texture support for wgpu backend, but discovered a fundamental architectural issue with how wgpu's PassExecutionContext collects and binds resources.

## What Was Attempted

1. ✅ Added default sampler creation
2. ✅ Created bind group layouts (5 bindings for group 0)
3. ✅ Updated WGSL forward shader with materials and textures
4. ⚠️ **BLOCKED:** Bind group lifetime issues

## The Problem

### Architecture Issue

wgpu's PassExecutionContext pattern (collect resources, then bind at draw time) has a critical issue:

```rust
// Current pattern in draw():
let bind_group_0 = device.create_bind_group(&descriptor);  // Created here
self.render_pass().set_bind_group(0, &bind_group_0, &[]);  // Bound here
// bind_group_0 drops here when scope ends!
self.render_pass().draw(...);  // But draw needs it here!
```

**Error:**
```
wgpu error: Validation Error
The current set RenderPipeline expects a BindGroup to be set at index 0
```

### Why It Happens

1. wgpu bind groups must live for the entire duration of the render pass
2. PassExecutionContext creates them inside `draw()` method
3. They go out of scope before the actual draw call executes
4. wgpu validation catches the dropped bind group

### Why Vulkan Works But wgpu Doesn't

**Vulkan:**
- Descriptor sets are GPU handles
- Binding records a command in the command buffer
- GPU references remain valid

**wgpu:**
- Bind groups are Rust objects with lifetimes
- Must stay alive until render pass ends
- Rust borrow checker enforces this

## Potential Solutions

### Option 1: Store Bind Groups in Context (Hard)

```rust
struct WgpuPassContext {
    render_pass: *mut (),
    backend: *mut WgpuBackend,
    // Store bind groups to keep them alive
    bind_groups: Vec<wgpu::BindGroup>,  // NEW
    uniform_buffers: Vec<...>,
}
```

**Problems:**
- Lifetime management complex
- Render pass already borrowed mutably
- May require unsafe code

### Option 2: Pre-create Bind Groups (Medium)

Create all possible bind group combinations ahead of time and reuse them.

**Problems:**
- Combinatorial explosion (many permutations)
- Not flexible for dynamic materials/textures
- Memory overhead

### Option 3: Refactor PassExecutionContext (Hard)

Change the entire execution model to create bind groups before the render pass starts.

**Problems:**
- Major architectural change
- Affects all backends
- Lots of code to refactor

### Option 4: Use wgpu Without PassExecutionContext (Medium)

Bypass the abstraction for wgpu and implement rendering directly.

**Problems:**
- Breaks abstraction
- Duplicate code
- Harder to maintain

## Current Workaround

**wgpu currently uses simple triangle shader:**
- No bind groups required
- Works for basic triangle rendering
- **Does NOT support:**
  - Materials
  - Textures
  - Lighting
  - Forward rendering

## Testing Results

```bash
# Triangle (simple shader) - ✅ Works
cargo run -- --scene scenes/triangle.toml --backend wgpu

# Textured cube (forward shader) - ❌ Fails
cargo run -- --scene scenes/textured_cube.toml --backend wgpu
# Error: Bind group not set
```

## What Actually Works

| Feature | Status |
|---------|--------|
| Basic triangle rendering | ✅ |
| Hardcoded vertices | ✅ |
| Color output | ✅ |
| Vertex buffers | ✅ |
| Uniform buffers | ❌ Lifetime issues |
| Textures | ❌ Requires uniforms |
| Materials | ❌ Requires uniforms |
| Forward rendering | ❌ Requires uniforms |

## Impact

**Practical:** Low
- wgpu→DX12/Metal/Vulkan paths don't work with textures
- **BUT** native Vulkan backend works perfectly ✅
- Most users will use native Vulkan on Linux
- Most users will use native DX12 on Windows (when implemented)

**wgpu backend is primarily useful for:**
- Web (WebGPU) - but textures don't work
- Quick prototyping - works for basic shapes
- Cross-platform fallback - limited functionality

## Recommended Actions

### Short Term
1. ✅ Keep wgpu with triangle shader (current state)
2. ✅ Document limitation clearly
3. ✅ Direct users to native Vulkan backend for full features

### Long Term
1. Refactor PassExecutionContext to pre-create bind groups
2. Or implement separate code path for wgpu
3. Or accept wgpu as "basic rendering only"

## Files Modified (Then Reverted)

- `src/backends/wgpu_backend/mod.rs` - Bind group changes
- `shaders/wgsl/forward.wgsl` - Material/texture bindings

**Status:** Reverted to working triangle shader

## Conclusion

wgpu texture support is **blocked by architectural issues** with PassExecutionContext lifetime management. Native Vulkan backend provides full texture support and is recommended for production use.

**Recommendation:** Use native Vulkan backend for textured rendering.

---

**Status:** wgpu remains at "basic triangle rendering only"  
**Vulkan:** Full texture support ✅  
**Priority:** Low (Vulkan backend covers main use case)

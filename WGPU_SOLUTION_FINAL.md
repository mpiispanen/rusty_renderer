# wgpu Implementation - Status and Solution

## Current Status

We've been debugging why wgpu fails with "BindGroup not set at index 0" even though we're calling `set_bind_group()`.

**Root cause identified:** Creating multiple mutable references to `wgpu::RenderPass` via unsafe pointer casts invalidates wgpu's internal state tracking.

## The Problem

Our architecture:
- `WgpuPassContext` stores `render_pass: *mut ()`
- Helper method creates `&mut wgpu::RenderPass<'a>` via unsafe cast
- Each call creates a NEW reference with fabricated lifetime
- wgpu's state tracking gets confused/invalidated

Tested and confirmed:
- Setting state on original reference, then using unsafe ref → FAILS
- Setting state via unsafe ref only → FAILS
- Setting state twice (both places) → FAILS

**Conclusion:** ANY use of unsafe pointer casts breaks wgpu.

## Why This Happens

`wgpu::RenderPass` is a stateful Rust object that tracks what's been set (pipeline, bind groups, vertex buffers). It relies on Rust's borrow checker for correctness.

When we bypass the borrow checker with unsafe casts, wgpu's assumptions break down.

## Solution Options

### Option 1: Abandon the Trait Abstraction for wgpu

Make wgpu use a completely different code path that doesn't go through `PassExecutionContext`.

**Pros:** 
- Clean, proper wgpu code
- No unsafe hacks
- Respects wgpu's design

**Cons:**
- Code duplication
- Two different execution models
- More maintenance

### Option 2: Store Proper Lifetimes

Change `WgpuPassContext` to store a proper reference:
```rust
pub struct WgpuPassContext<'a> {
    render_pass: &'a mut wgpu::RenderPass<'a>,
    backend: &'a mut WgpuBackend,
}
```

**Problem:** Can't borrow backend mutably while also borrowing render pass mutably.

**Solution:** Split backend state - store bind groups separately, pass them in.

### Option 3: Two-Phase with Deferred Binding

Current two-phase:
1. Prepare - create bind groups
2. Execute - bind and draw

New three-phase:
1. Prepare - create bind groups
2. Collect - passes report what to bind/draw
3. Execute - set everything on original ref and draw

**Pros:** Clean separation
**Cons:** Bigger refactor

### Option 4: Interior Mutability

Use `RefCell` to allow multiple borrows:
```rust
pub struct WgpuPassContext {
    render_pass: Rc<RefCell<wgpu::RenderPass<'static>>>,
    backend: *mut WgpuBackend,
}
```

**Problem:** Lifetimes - render pass can't be 'static

## Recommended Solution

**Hybrid approach:**

1. Keep the trait abstraction for Vulkan/DirectX  
2. For wgpu, have `PassExecutionContext::draw()` be a no-op
3. Add wgpu-specific API to `WgpuPassContext`:
   ```rust
   impl WgpuPassContext {
       pub fn wgpu_draw_collected(&mut self, render_pass: &mut wgpu::RenderPass) {
           // Set vertex buffers that were collected
           // Issue draw call
       }
   }
   ```
4. In `execute_graph()`, after pass callbacks:
   ```rust
   // Vertex buffers collected by now
   for buffer in context.vertex_buffers {
       render_pass.set_vertex_buffer(...);
   }
   render_pass.draw(...);
   ```

This respects both:
- The trait abstraction (for Vulkan/DirectX)
- wgpu's borrowing requirements (for wgpu)

## Implementation Plan

1. Make `WgpuPassContext::draw()` just collect draw parameters, don't actually draw
2. After pass callbacks finish, set vertex buffers on original `render_pass`
3. Call `render_pass.draw()` with collected parameters
4. No more unsafe pointer casts to render pass!

## Files to Modify

- `src/backends/wgpu_backend/mod.rs` - execute_graph() and WgpuPassContext
- Maybe `src/passes/forward.rs` - if we need wgpu-specific behavior

## Next Steps

1. Implement the deferred draw approach
2. Test with textured cube
3. Document the wgpu-specific execution model
4. Move forward with features using Vulkan primarily

# WGPU Mystery - Partially Solved

## Problem
When running the vertex_buffer_triangle example with wgpu backend, we get:
```
wgpu error: Validation Error
  In RenderPass::end
    In a draw command, kind: Draw
      Currently set RenderPipeline with 'Simple Vertex Color Pipeline' label requires vertex buffer 0 to be set
```

## What We've Verified
✓ Vertex buffer is created with VERTEX usage  
✓ Buffer has correct size (144 bytes = 3 vertices * 48 bytes)  
✓ Pipeline expects vertex buffer at slot 0 with correct attributes  
✓ Shader expects locations 0-3 (position, normal, uv, color)  
✓ set_vertex_buffer(0, ...) is being called  
✓ draw() is being called immediately after  

## Root Cause
The WgpuPassContext stores the render pass as a raw pointer and creates new `&mut RenderPass<'a>` references via unsafe casts. Each call to `render_pass()` creates a reference with an arbitrary lifetime.

**This violates Rust's safety guarantees and likely confuses wgpu's internal command recording.**

## The Real Issue
When we cast from `*const dyn Buffer` to `*const c_void`, we lose the vtable pointer (fat pointer becomes thin pointer). This is a fundamental architectural issue with the current trait design.

## Solution Options

### Option 1: Use wgpu-specific API (current workaround)
- Have passes detect wgpu backend and call wgpu-specific methods
- Bypasses the trait's pointer conversion
- **Status**: Implemented but STILL FAILING due to render pass pointer issues

### Option 2: Fix the trait to preserve fat pointers
- Change PassExecutionContext::bind_vertex_buffer to take `&dyn Buffer` instead of `*const c_void`
- Requires changing the trait interface
- **Status**: Not attempted (breaking change)

### Option 3: Restructure wgpu backend to not use pointer casts
- Don't store render pass as pointer in WgpuPassContext
- Pass render pass differently (maybe via RefCell or similar)
- **Status**: Not attempted (large refactor)

### Option 4: Use a completely different architecture for wgpu
- Don't try to fit wgpu into the Vulkan-style trait
- Have wgpu-specific render graph execution
- **Status**: Not attempted (very large change)

## Next Steps
We need to try Option 3 - fix the render pass pointer issue by not using unsafe pointer casts.

The render pass lifetime issue is the blocker, not the vertex buffer binding itself.

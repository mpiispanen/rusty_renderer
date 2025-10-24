# wgpu Bind Group Mystery - SOLVED!

## Root Cause

Creating multiple mutable references to `wgpu::RenderPass` via unsafe pointer casts **invalidates wgpu's internal state tracking**.

When we:
1. Set pipeline/bind groups on the original `&mut RenderPass`
2. Create `WgpuPassContext` storing `*mut ()` pointer
3. Later dereference as `&mut *render_pass_ptr`

Step 3 creates a NEW mutable reference, and wgpu's state (pipeline, bind groups) gets invalidated or reset!

## Evidence

We tested:
- Setting bind groups on original reference: ❌ Still fails
- Setting bind groups via unsafe pointer: ❌ Still fails  
- Setting bind groups TWICE (both places): ❌ Still fails!

The act of creating the unsafe reference invalidates everything.

## The Fix

We MUST set pipeline, bind groups, AND vertex buffers all on the SAME original reference, with NO unsafe pointer casts in between.

But there's a problem: We don't know which vertex buffers to use until the pass execute() callback runs!

## Solution: Two-Step Execution

1. **Step 1: Collect Phase** - Pass callbacks collect vertex buffers (don't bind yet)
2. **Step 2: Bind Phase** - Set everything on original render_pass reference
3. **Step 3: Draw** - Call draw (can use unsafe ref for this, it's just a command)

Changes needed:
- WgpuPassContext::bind_vertex_buffer() - just collect, don't bind
- After all passes execute, set vertex buffers on original ref
- Then call draw on original ref (or via unsafe, since draw doesn't affect state)

This matches wgpu's design: All state must be set through a continuous borrow.

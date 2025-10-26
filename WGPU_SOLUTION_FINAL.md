# wgpu Backend - Final Solution

## Status
✅ **Working for CI and single-frame renders**  
❌ **Not suitable for interactive/continuous rendering on AMD+Linux**

## The Problem
wgpu's Vulkan backend on AMD+Linux doesn't properly implement frame synchronization:
- `PresentMode::Fifo` should block on `present()` until GPU completes
- It doesn't block, so swapchain images stay "in flight"
- With 3 swapchain images, can render 4-5 frames before exhaustion
- No way to wait for GPU without blocking event loop

## What Works
✅ Single frame rendering (for CI tests)  
✅ Headless rendering  
✅ Short animations (< 5 frames)  
✅ Resource management (no leaks)  
✅ Proper bind group lifecycle

## What Doesn't Work  
❌ Continuous rendering (> 5 frames)  
❌ Interactive applications  
❌ Games/real-time rendering

## Why We Can't Fix It
wgpu API limitations:
1. `device.poll(Wait)` blocks event loop → deadlock
2. `device.poll(Poll)` doesn't wait → doesn't help
3. No async/non-blocking wait API
4. Can't access underlying Vulkan fences

## Recommendation
**Use Vulkan or DirectX backends for production.**

wgpu is useful for:
- Cross-platform prototyping
- WebGPU targets
- CI rendering tests (single frames)
- macOS/Metal (where it works better)

## Implementation Notes
Our wgpu code is correct per spec:
- Proper acquire → render → present → drop flow
- Bind groups cleared after present
- No resource leaks
- Matches official wgpu examples

The limitation is in wgpu's Vulkan backend, not our code.

---
**For this project:** Use Vulkan (Linux) or DirectX (Windows) for real rendering.

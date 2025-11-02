# Vulkan Rendering Debug Session - Part 3
## Date: 2025-11-02

### Issue
Both Vulkan and DirectX backends render only a black screen after migrating to render graph resource management.

### Root Cause Found
**The render pass was not being begun before executing passes!**

The migration to render graph removed the render pass begin/end logic but forgot to add it back. After adding:
```rust
unsafe {
    device.cmd_begin_render_pass(command_buffer, &render_pass_info, vk::SubpassContents::INLINE);
}
// ... execute passes ...
unsafe {
    device.cmd_end_render_pass(command_buffer);
}
```

**However, the output is still black!** This suggests there are additional problems.

### Investigation So Far

1. **Confirmed the rendering pipeline is executing:**
   - 36 vertices are being drawn for cube
   - 3 vertices for triangle  
   - Shaders are compiled successfully
   - No validation errors
   - Command buffers are being submitted
   - Draw calls succeed
   - Render pass is now being begun/ended

2. **Fixed Y-axis flip for Vulkan:**
   - Modified `perspective()` function to negate Y component for Vulkan coordinate system
   - This matches Vulkan's inverted Y-axis compared to OpenGL

3. **Fixed missing draw call in TrianglePass:**
   - Triangle pass callback now calls `context.draw(3, 1, 0, 0)`
   - Draw succeeds without error

4. **Fixed missing render pass:**
   - Added render pass begin before executing passes
   - Added render pass end after all passes complete

### Current State

**Screenshot analysis:**
- Triangle and cube scenes both still produce black output (mean value: ~0.006)
- Draw calls are being issued successfully
- Render pass is active

**Remaining possible causes:**
1. **Viewport/Scissor not set** - May still be at default (0,0,0,0)
2. **Shader issues** - Shaders may not be compiled correctly or have bugs
3. **Pipeline state** - Something wrong with rasterizer state, blend state, etc.
4. **Depth buffer** - Could be rejecting all fragments
5. **Color attachment format mismatch**

### Code Changes Made

**src/app.rs:**
- Added Y-flip to perspective matrix for Vulkan

**src/passes/triangle_pass.rs:**
- Added `context.draw(3, 1, 0, 0)` call in execute()

**src/backends/vulkan/mod.rs:**
- Added render pass begin/end around pass execution (lines ~3120-3170)

### Next Steps

1. **Check viewport and scissor:**
   - Add logging to verify viewport/scissor are being set
   - Ensure they match the swapchain extent

2. **Verify shader output:**
   - Check if shaders are actually outputting colors
   - Try a simpler shader that just outputs a solid color

3. **Check pipeline state:**
   - Log all pipeline creation parameters
   - Verify depth test settings, blend mode, etc.

4. **Use RenderDoc:**
   - Capture a frame to inspect GPU state
   - See exactly what's being rendered

### Session Status
- Vulkan backend: Still black screen despite fixes
- DirectX backend: Not tested yet in this session
- Need to continue debugging why geometry isn't visible

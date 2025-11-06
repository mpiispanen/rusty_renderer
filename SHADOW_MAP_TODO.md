# Shadow Mapping Implementation Status

## Current Issue
Shadow mapping is temporarily disabled due to architectural limitations in the render graph backend.

## Root Cause
The current Vulkan backend uses a single hardcoded render pass for all rendering operations. The shadow map pass requires:
- Its own depth-only render pass (no color attachments)
- A separate framebuffer with 1024x1024 extent (different from swapchain)
- Depth texture that can be sampled in subsequent passes

When we tried to execute the shadow pass within the main render pass, it caused visual artifacts because:
1. The shadow pass attempted to render to a 1024x1024 depth texture
2. But was executed within a render pass configured for swapchain (1280x720) with color+depth
3. This caused geometry to be rendered to the wrong location/texture

## Required Changes for Proper Shadow Mapping

### 1. Multi-Render Pass Support
- Modify `execute_graph` to support multiple Vulkan render passes per frame
- Each pass should create its own `vkBeginRenderPass`/`vkEndRenderPass`
- Passes should be grouped by compatible renderpass/framebuffer requirements

### 2. Dynamic Framebuffer Creation
- Create framebuffers dynamically based on pass outputs
- Shadow pass needs: framebuffer with 1024x1024 depth attachment only
- Forward pass needs: framebuffer with swapchain color + depth attachments

### 3. Image Layout Transitions
- Properly transition shadow map from DEPTH_STENCIL_ATTACHMENT (write) to SHADER_READ_ONLY (sample)
- Handle synchronization between render passes

### 4. Viewport/Scissor Per Pass
- Currently all passes use swapchain extent for viewport
- Shadow pass should use 1024x1024 viewport

## Temporary Solution
Shadow mapping code exists in:
- `src/passes/shadow_map.rs` - Shadow pass implementation
- `shaders/hlsl/shadow_map.hlsl` - Shadow shaders  
- `shaders/hlsl/forward_simple.hlsl` - Has shadow sampling code

The shadow pass creation is commented out in `src/app.rs` line 252-282.

## When Re-enabling
1. Implement multi-render pass support in backends
2. Uncomment shadow pass creation in `src/app.rs`
3. Test that artifact is gone and shadows render correctly
4. Test with more complex scenes (not just single triangle)


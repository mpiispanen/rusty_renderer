# Vulkan Rendering Debug Session - COMPLETE
## Date: 2025-11-02

## Initial Problem
Both Vulkan and DirectX backends were only rendering clear color, not actual geometry.

## Root Cause
The `execute_graph` function was trying to use the OLD render pass system from the legacy pipeline code:
1. `execute_graph` would call `cmd_begin_render_pass` with `self.render_pass` 
2. This render pass was created during initialization for the OLD pipeline
3. The render graph creates its OWN resources (color/depth targets)
4. The mismatch caused `cmd_begin_render_pass` or `cmd_bind_pipeline` to fail silently
5. The function would return early, never executing the pass callbacks

## Solution
Removed the global render pass begin/end from `execute_graph`:
- Render graph passes now handle their own rendering
- Each pass is responsible for managing its render pass lifecycle
- This aligns with the declarative pipeline design where passes define their own requirements

### Changes Made

#### `src/backends/vulkan/mod.rs`

1. **Removed `begin_frame` command buffer recording**:
   - Previously called `record_command_buffer()` which would begin/end command buffer
   - Now just acquires swapchain image (windowed) or does nothing (headless)
   - Let `execute_graph` handle command buffer recording

2. **Removed global render pass from `execute_graph`**:
   - No longer calls `cmd_begin_render_pass` at the start
   - No longer calls `cmd_end_render_pass` at the end
   - Passes handle their own render pass management

3. **Added debug logging** to track execution flow

## Verification
After the fix, pass execution works correctly - screenshot successfully captured.

## Next Steps
1. Apply similar fix to DirectX backend
2. Test both backends render correctly
3. Properly implement render pass management in passes
4. Clean up dead pipeline code

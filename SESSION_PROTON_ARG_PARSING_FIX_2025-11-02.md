# DirectX Proton Argument Parsing Fix - Session Summary
## Date: 2025-11-02

### Issue
The `run_with_proton.sh` script was failing silently with exit code 1. The application would start but crash during argument parsing when scene paths were passed.

### Investigation
1. The Windows binary wasn't showing any logs via Wine stderr/stdout
2. Added debug logging to file (`rusty_renderer_debug.log`) revealed:
   - App started successfully  
   - Logging initialized
   - Arguments parsed
   - **CRASH in `upload_to_buffer` with error `0x80004005` (E_FAIL)**

3. VKD3D warnings showed: "A command list using this allocator is in the recording state"

### Root Cause
DirectX 12 command list/allocator lifecycle violation:

1. `begin_frame()` resets the command allocator and command list (puts it in recording state)
2. `execute_graph()` calls `allocate_graph_resources()`  
3. `allocate_graph_resources()` uploads initial buffer data via `upload_to_buffer()`
4. `upload_to_buffer()` tries to reset the command allocator/list **while still in recording state**
5. DirectX 12 rule: Cannot reset a command allocator while any associated command list is in recording state

### Solution
Modified `upload_to_buffer()` in `src/backends/directx/dx12_impl.rs`:

```rust
unsafe {
    // Close the command list if currently recording
    let _ = cmd_list.Close(); // Ignore error if already closed
    
    // Wait for any pending GPU operations
    if fence.GetCompletedValue() < self.fence_value - 1 {
        fence.SetEventOnCompletion(self.fence_value - 1, self.fence_event)?;
        WaitForSingleObject(self.fence_event, INFINITE);
    }

    // Now safe to reset
    cmd_allocator.Reset()?;
    cmd_list.Reset(cmd_allocator, None)?;

    // Perform the upload copy
    cmd_list.CopyBufferRegion(...);
    cmd_list.Close()?;
    
    // Execute and wait
    command_queue.ExecuteCommandLists(&[...]);
    command_queue.Signal(fence, fence_value)?;
    // ... wait for completion ...

    // Reset command list again for caller
    // (execute_graph expects recording state after resource allocation)
    cmd_allocator.Reset()?;
    cmd_list.Reset(cmd_allocator, None)?;
}
```

### Additional Fixes
1. Updated `run_with_proton.sh` to default `RUST_LOG=debug` instead of `info`
2. Added debug logging in `upload_to_buffer` for both staging and CPU-accessible buffer paths
3. Updated `main.rs` to print full error chain with `{e:?}` instead of `{e}`

### Testing
```bash
# Now works:
./run_with_proton.sh --headless --max-frames 1 --scene triangle
./run_with_proton.sh --headless --max-frames 1 --scene cube --screenshot test.png
./run_with_proton.sh --headless --max-frames 1 --scene scenes/gltf_textured.toml
```

All exit with code 0 and produce correct output.

### Files Modified
- `src/backends/directx/dx12_impl.rs` - Fixed command list lifecycle in `upload_to_buffer()`
- `src/main.rs` - Enhanced error logging
- `run_with_proton.sh` - Updated default RUST_LOG level

### Notes
This is a temporary fix. The proper solution would be to use a separate upload command allocator/list dedicated to resource uploads, avoiding interference with the main rendering command list. This should be implemented as part of the resource management refactor (issue #87).

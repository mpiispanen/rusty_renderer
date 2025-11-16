# DirectX 12 Rendering Problem Analysis

**Date:** November 14, 2025  
**Status:** DX backend produces black/artifact output with GPU faults under Proton

## Executive Summary

The DirectX 12 backend is experiencing critical rendering failures when run through Proton/VKD3D-Proton. While Vulkan backend renders correctly, DX produces either black output or clear color with white artifacts, accompanied by GPU memory access violations and synchronization errors.

## Current State

### Working
- ✅ Vulkan backend renders damaged helmet correctly (both headless and windowed)
- ✅ DX compiles and initializes without errors
- ✅ DX creates resources (buffers, textures, pipelines) successfully
- ✅ Basic command list recording works

### Broken
- ❌ DX rendering produces black output or clear color only
- ❌ GPU VM faults (address 0x00000000) under Proton
- ❌ Command allocator synchronization errors
- ❌ Device lost (VK_ERROR_DEVICE_LOST) after GPU fault
- ❌ Texture data may not be properly bound or visible to shaders

## Error Patterns

### 1. GPU Memory Access Violation (GPUVM Fault)

```
radv: GPUVM fault detected at address 0x00000000.
GCVM_L2_PROTECTION_FAULT_STATUS: 0x201430
	 CLIENT_ID: (SQC (data)) 0xa
	 MORE_FAULTS: 0
	 WALKER_ERROR: 0
	 PERMISSION_FAULTS: 3
	 MAPPING_ERROR: 0
	 RW: 0
```

**Analysis:**
- GPU shader (SQC = Shader Quad Cache) trying to read from address 0x00000000
- NULL pointer access from shader side
- Indicates missing or unbound resource (texture/buffer)
- PERMISSION_FAULTS: 3 suggests multiple access attempt failures

### 2. Command Allocator Synchronization Errors

```
err:vkd3d-proton:d3d12_command_allocator_Reset: There are still 1 pending command lists awaiting execution from command allocator
err:vkd3d-proton:d3d12_command_allocator_Reset: There are still 307 pending command lists awaiting execution from command allocator
```

**Analysis:**
- Command allocator being reset while command lists are still in flight
- Fence synchronization not properly waiting for GPU completion
- Multiple fence values being tracked but not properly synchronized:
  - `self.fence_value` - current fence value counter
  - `self.main_allocator_fence_value` - fence value for main allocator's last use
  - `self.upload_allocator_fence_value` - fence value for upload allocator's last use

### 3. Device Lost Cascade

After GPU fault:
```
err:vkd3d-proton:d3d12_command_queue_execute: Failed to submit queue(s), vr -4.
warn:vkd3d-proton:d3d12_device_mark_as_removed: Device is lost (reason 0x887a0005, "VK_ERROR_DEVICE_LOST")
```

**Analysis:**
- Initial GPU fault causes device removal
- All subsequent operations fail with VK_ERROR_DEVICE_LOST
- Cannot recover without device reset

## Root Cause Hypotheses

### Hypothesis 1: Descriptor Heap/SRV Binding Issue (MOST LIKELY)

**Evidence:**
- GPU accessing NULL address suggests unbound texture
- Shader trying to sample texture that doesn't exist in descriptor heap
- `descriptor_heap_offset` management may have race conditions

**Code Locations:**
- `create_texture()` at line 2911-3050: Creates SRV and increments descriptor_heap_offset
- Texture upload at line 3160-3280: Uses separate upload command list
- Pass execution: Binds descriptor heaps and root signature

**Potential Issues:**
1. Descriptor heap not properly bound to command list before draw
2. Root descriptor tables pointing to wrong heap offsets
3. SRV created but GPU descriptor handle not properly passed to shader
4. Texture upload fence synchronization - texture may not be ready when sampled

### Hypothesis 2: Command Allocator Lifecycle Mismatch

**Evidence:**
- Errors about pending command lists when resetting allocator
- Two separate allocators (main and upload) with independent fence tracking

**Code Locations:**
- `begin_frame()` at line 1073: Waits for main_allocator_fence_value before reset
- `upload_texture_data()` at line 3177-3190: Waits for upload_allocator_fence_value before reset
- `end_frame()` at line 1103: Signals fence and updates main_allocator_fence_value

**Potential Issues:**
1. Upload operations happening async while main rendering tries to use resources
2. Fence values not properly tracking all command list submissions
3. Multiple command lists using same allocator without proper synchronization
4. Upload command list may not have completed before main rendering samples texture

### Hypothesis 3: Resource State Transition Errors

**Evidence:**
- Textures created in COMMON state
- Transitioned to COPY_DEST for upload
- Transitioned to PIXEL_SHADER_RESOURCE for sampling
- But main render pass may need additional barriers

**Code Locations:**
- Texture creation: Initial state D3D12_RESOURCE_STATE_COMMON (line 2979)
- Upload barriers: COMMON → COPY_DEST → PIXEL_SHADER_RESOURCE (lines 3194-3264)
- Main render: Assumes texture in correct state

**Potential Issues:**
1. Texture state not properly synchronized between upload and render
2. Missing barriers in execute_graph before binding textures
3. State tracking may not match actual GPU state
4. Render pass may be using texture before upload completes

### Hypothesis 4: Buffer/Vertex Data Not Properly Bound

**Evidence:**
- Output shows clear color, suggesting fragment shader may not even run
- Model may not be drawing at all (no geometry visible)

**Code Locations:**
- Buffer creation at line 2579
- Vertex/index buffer binding in pass execution

**Potential Issues:**
1. Vertex buffers not bound or contain invalid data
2. Index buffer pointing to wrong location
3. Draw call parameters incorrect (vertex count, instance count)
4. Pipeline input assembly state mismatch with vertex data

## Synchronization Architecture

### Current Fence Tracking
```
fence_value:                     u64  // Global counter, increments after each signal
main_allocator_fence_value:      u64  // Fence value when main allocator was last used
upload_allocator_fence_value:    u64  // Fence value when upload allocator was last used
```

### Fence Flow in Normal Frame
1. **begin_frame()**: Wait for main_allocator_fence_value, reset main allocator
2. **execute_graph()**: Record rendering commands
3. **end_frame()**: Close list, execute, signal fence_value, update main_allocator_fence_value

### Fence Flow in Texture Upload
1. Wait for upload_allocator_fence_value
2. Reset upload allocator
3. Record upload commands
4. Execute upload command list
5. Signal fence_value, update upload_allocator_fence_value
6. **MISSING**: Wait for upload to complete before main render uses texture?

## Comparison with Working Vulkan Backend

The Vulkan backend successfully renders the same content, suggesting:
1. HLSL shaders are functionally correct (compiled from same source intent)
2. Scene data (vertex buffers, textures) are valid
3. The issue is DX12-specific resource binding or synchronization

## Key Questions to Answer

1. **Are textures actually uploaded to GPU?**
   - Add validation: Check GPU memory after upload
   - Verify texture resource has data before sampling

2. **Are descriptor heaps properly bound?**
   - Check SetDescriptorHeaps call before draw
   - Verify root descriptor table indices match heap layout

3. **Is geometry being drawn at all?**
   - Check draw call parameters
   - Verify vertex/index buffers contain valid data
   - Validate pipeline state has correct input layout

4. **Are fences actually synchronizing?**
   - Add logging of fence values at each wait/signal
   - Check if GetCompletedValue matches expected value
   - Verify upload completes before render samples texture

5. **Are resource states correct?**
   - Track expected vs actual resource states
   - Add validation barriers
   - Check if Proton/VKD3D properly implements state tracking

## Diagnostic Steps Needed

### Immediate Actions
1. **Add comprehensive logging:**
   - Fence values at every wait/signal point
   - Descriptor heap binding confirmation
   - Draw call parameters
   - Resource state transitions

2. **Validate texture upload:**
   - Add fence wait after upload before returning from upload_texture_data
   - Verify texture resource has non-zero GPU address
   - Check SRV descriptor is valid

3. **Check descriptor heap binding:**
   - Ensure SetDescriptorHeaps called before SetGraphicsRootDescriptorTable
   - Verify heap has enough space for all descriptors
   - Validate GPU handles are within heap range

4. **Verify draw parameters:**
   - Log vertex count, index count, instance count
   - Check vertex buffer is bound
   - Verify index buffer format and offset

### Testing Methodology
1. **Minimal reproduction:**
   - Try rendering cube with no texture (solid color)
   - Add texture one at a time
   - Isolate which resource binding fails

2. **Native Windows testing:**
   - Build and run on actual Windows with native DX12
   - Determine if issue is Proton-specific or general DX bug

3. **Validation layer:**
   - Enable DX12 debug layer (if available under Proton)
   - Check for validation errors

## Files Involved

### Primary
- `src/backends/directx/dx12_impl.rs` - Main DX12 implementation (3900+ lines)
  - Line 1073: begin_frame()
  - Line 1103: end_frame()
  - Line 2198: execute_graph()
  - Line 2579: create_buffer()
  - Line 2911: create_texture()
  - Line 3160-3280: upload_texture_data()

### Supporting
- `src/backends/directx/mod.rs` - DirectX backend interface
- `shaders/hlsl/forward.hlsl` - HLSL shader code
- `run_with_proton.sh` - Proton test script

## Historical Context

Previous attempts have addressed:
- Render pass separation (working)
- Y-axis coordinate fixes (working in Vulkan)
- Default texture creation (added but didn't fix issue)
- Multiple synchronization attempts (added sleeps, multiple fences)

The core issue remains: **GPU cannot access texture data during rendering.**

## Recommended Next Steps

1. **Add explicit fence wait after texture upload** - Ensure texture data is available before render
2. **Validate descriptor heap binding sequence** - Confirm heaps bound before root tables set
3. **Check for resource aliasing** - Ensure upload and render don't conflict on same resource
4. **Test with simpler scene** - Single triangle with solid color, no textures
5. **Enable all available validation** - Catch errors earlier in the pipeline

## Success Criteria

- No GPU VM faults
- No command allocator reset errors  
- No device lost errors
- Rendered output matches Vulkan output
- Performance comparable to Vulkan (no artificial delays/sleeps)

# DirectX 12 Rendering Debug Analysis

## Current Status (Nov 15, 2025)

### Symptoms
1. **DX Proton (vkd3d-proton) - Device Lost / GPUVM Fault**
   - Errors: `d3d12_command_allocator_Reset: There are still X pending command lists awaiting execution`
   - GPU VM fault: `GPUVM fault detected at address 0x00000000`
   - `GCVM_L2_PROTECTION_FAULT_STATUS: 0x201430` with `PERMISSION_FAULTS: 3`
   - Device lost: `VK_ERROR_DEVICE_LOST`
   - Output: Clear color with white-ish artifacts OR black screen

2. **Vulkan Backend (for comparison)**
   - Works correctly in headless mode
   - Draws damaged helmet model properly 
   - Textures load and display correctly

### Key Error Pattern
```
d3d12_command_allocator_Reset: There are still X pending command lists awaiting execution
radv: GPUVM fault detected at address 0x00000000
PERMISSION_FAULTS: 3 (indicates read/write permission violations)
VK_ERROR_DEVICE_LOST
```

## Root Cause Hypotheses

### 1. **Command List Synchronization Issue** (Most Likely)
The error "There are still X pending command lists awaiting execution" suggests:
- Command allocator is being reset while command lists are still in flight
- Not waiting for GPU to complete execution before resetting allocator
- Possible fence synchronization bug

**Evidence:**
- Error count increases: "still 1 pending" → "still 307 pending" → "still 308 pending"
- Suggests cumulative synchronization failure
- This directly violates D3D12 specification rules

### 2. **Invalid GPU Memory Access** (Likely Related)
The GPUVM fault at address 0x0 strongly suggests:
- Null pointer dereference in GPU shader or descriptor
- Descriptor heap not properly bound
- Buffer/texture resource freed or unmapped while GPU is accessing it
- Invalid descriptor table binding

**Evidence:**
- Fault at address 0x00000000 (null)
- PERMISSION_FAULTS: 3 (likely read + write attempt on invalid memory)
- This causes the device loss

### 3. **Descriptor Heap/Binding Issues**
Based on code inspection:
- Multiple command allocators exist but unclear if properly synchronized
- Descriptor heaps may not be bound correctly for each frame
- GPU-visible descriptor heaps might not contain texture descriptors

### 4. **Resource State Transitions**
- Resources might not be in correct state when accessed
- Barrier synchronization incomplete
- Texture resources might be in wrong state when shader tries to sample

## Code Areas of Concern

### 1. Command Allocator Management
**Location:** `src/backends/directx/dx12_impl.rs`

Current pattern:
```rust
// Per-frame command allocator
command_allocators: Vec<ID3D12CommandAllocator>
current_frame: usize

// Reset happens at begin_frame
allocator.Reset()?;
```

**Problem:** Not verifying GPU has finished with allocator before reset.

### 2. Fence Synchronization
**Location:** `dx12_impl.rs` - `wait_for_frame` and frame management

Current has fences but unclear if:
- Waiting for correct fence value
- All command lists using allocator have completed
- Fence values properly incremented per submission

### 3. Screenshot Command List
**Issue:** Screenshot path creates separate command list but may share allocator.
- Not clear if properly synchronized
- May cause "pending command lists" error

### 4. Descriptor Heap Binding
**Location:** `execute_pass` and draw call setup

Questions:
- Are descriptor heaps set on command list before draws?
- Are texture SRVs in GPU-visible heap?
- Is descriptor table root parameter set correctly?

### 5. Texture Resource State
**Location:** Texture creation and usage

Questions:
- Are textures transitioned to PIXEL_SHADER_RESOURCE before sampling?
- Are upload buffers properly synchronized?
- Are texture descriptors created in correct heap?

## Validation Layers

### D3D12 Debug Layer
- Partially implemented in code
- Enabled via `enable_validation` parameter
- Should provide detailed warnings about synchronization and API usage
- **Status: Need to verify it's actually working under vkd3d-proton**

### VKD3D-Proton Debug Output
- Already shows errors but not detailed enough
- May need additional environment variables:
  - `VKD3D_DEBUG=warn,err`
  - `VKD3D_SHADER_DEBUG=warn`
  - `WINEDEBUG=+d3d12`

## Next Steps for Debugging

### Phase 1: Enable Full Validation
1. ✅ Verify D3D12 debug layer is enabled
2. ✅ Add VKD3D debug environment variables
3. Run with validation and capture full log
4. Look for specific API violations

### Phase 2: Fix Command Allocator Synchronization
1. Add explicit fence waits before allocator reset
2. Ensure fence values properly track GPU progress
3. Verify no command lists are in recording state when resetting allocator
4. Consider separate allocators for screenshot path

### Phase 3: Fix Descriptor Issues
1. Verify descriptor heap binding in execute_pass
2. Ensure texture descriptors in GPU-visible heap
3. Add descriptor table binding validation
4. Check root signature matches shader expectations

### Phase 4: Fix Resource States
1. Add proper barriers for texture transitions
2. Ensure textures in PIXEL_SHADER_RESOURCE state before sampling
3. Verify upload synchronization
4. Check depth buffer states

## Technical Details

### GPUVM Fault Breakdown
```
GCVM_L2_PROTECTION_FAULT_STATUS: 0x201430
- CLIENT_ID: SQC (data) 0xa  = Shader fetch unit
- PERMISSION_FAULTS: 3        = Read + Write violations
- MAPPING_ERROR: 0            = Address was mapped (not unmapped memory)
- RW: 0                       = Read access
```

This indicates shader tried to read from address 0x0 which is mapped but has no permission.
Likely cause: Null descriptor or incorrect descriptor table.

### D3D12 Command Allocator Rules
From D3D12 spec:
1. Command allocator can only be reset when all command lists using it have completed on GPU
2. Must use fence to track GPU progress
3. Violating this causes undefined behavior and likely device loss

### vkd3d-proton Architecture
- Translates D3D12 to Vulkan
- Our GPUVM fault is actually in Vulkan/RADV driver
- Indicates the Vulkan resources are being accessed incorrectly
- This is a translation-level bug OR our D3D12 usage is wrong

## Comparison with Vulkan

Vulkan backend works, so compare:
- How are textures created and bound?
- How are descriptor sets updated?
- How is synchronization handled?
- What's different about command buffer management?

## Environment
- OS: Linux (Fedora)
- GPU: AMD (using RADV driver)
- D3D12: vkd3d-proton (translates to Vulkan)
- Proton: Running Windows binary under Wine

This creates a complex debugging scenario as issues could be:
1. Our D3D12 code
2. vkd3d-proton translation bugs
3. Vulkan driver issues
4. Interaction between all layers

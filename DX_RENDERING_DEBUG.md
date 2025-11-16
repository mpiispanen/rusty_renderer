# DirectX Rendering Debug Report

## Current Status (2025-11-15)

### Working
- ✅ Vulkan backend renders correctly (damaged helmet with texture)
- ✅ DirectX builds successfully for Windows (x86_64-pc-windows-gnu)
- ✅ DirectX runs under Proton without crashes in headless mode
- ✅ Logging now works properly under Wine/Proton (logs to file and stdout/stderr)
- ✅ Texture loading and upload completes successfully
- ✅ SRV descriptors are created and bound correctly
- ✅ Screenshot capture completes (produces 20KB PNG file)

### Issues

#### 1. Output is Black / Incorrect
**Symptom:** DirectX renders to an all-black image or shows artifacts instead of the textured model

**Log Evidence:**
```
[2025-11-15 22:59:05.278] Executing forward simple pass (36 vertices, 36 indices, has_texture: true)
[2025-11-15 22:59:05.318] Set descriptor table at root parameter 3, t0 handle: 0x300000000
[2025-11-15 22:59:05.320] Rendered 1 frames
```

**What's Working:**
- Scene loads correctly (36 vertices, 36 indices)
- Texture uploads successfully (256x256, 262144 bytes)
- SRV created at GPU handle 0x300000000
- Descriptor table set at root parameter 3
- Vertex/index buffers created and bound
- Camera matrices calculated correctly
- No D3D12 errors or validation warnings

**Potential Causes:**
1. **Rendering Pipeline Issue**
   - Draw calls may not be executing properly
   - Command list may not be submitted correctly
   - Render target may not be cleared or bound correctly
   
2. **Synchronization Issue**
   - 10-second delay in frame capture suggests GPU sync problem
   - Fence waits may be incorrect
   - Command allocator reuse timing issues

3. **Shader/Pipeline State**
   - Root signature mismatch
   - Pipeline state not set correctly before draw
   - Descriptor heap not bound when drawing
   
4. **Resource States**
   - Texture may be in wrong state during rendering
   - Render target transitions incorrect
   - Missing resource barriers

#### 2. Performance Issue - 10 Second Delay
**Symptom:** Frame capture takes 10 seconds (from 22:59:05 to 22:59:15)

**Code Location:** `capture_frame()` in dx12_impl.rs:1087-1266

**Potential Causes:**
- GPU timeout during fence wait
- Command list not completing
- Resource state conflicts
- Multiple command lists competing for same resources

## Recent Changes
- Removed malformed debug code that caused compilation errors
- Fixed unclosed delimiter in DirectXPassContext
- Added stderr to logging output
- Fixed logging setup to work under Wine/Proton

## Investigation Results (2025-11-15 23:03 UTC)

### ✅ Verified Working Components
1. **Draw Calls** - `DrawIndexedInstanced` is called with correct parameters (36 indices, 1 instance)
2. **Render Target** - RTV is cleared with color [0.1, 0.1, 0.2, 1.0] and set correctly (ptr: 0x16cbf00)
3. **Depth Buffer** - DSV is cleared and bound
4. **Viewport/Scissor** - Set to full framebuffer (1280x720)
5. **Vertex Buffer** - Bound at binding 0, stride 48 bytes
6. **Index Buffer** - Bound with U32 format
7. **Push Constants** - 192 bytes (3x 4x4 matrices) set at root parameter 2
8. **Lighting Uniforms** - Bound at binding 0
9. **Texture** - Albedo texture uploaded (256x256) and bound
10. **Command List** - Submitted and executed
11. **Descriptor Heaps** - CBV/SRV/UAV heap set before draw calls

### ❓ Remaining Issues to Investigate

Since all the rendering pipeline components appear to be working correctly, the black output suggests:

1. **Shader Execution Problem**
   - Shaders might not be executing properly under vkd3d-proton
   - Possible DXIL vs SPIR-V compilation issue
   - Root signature mismatch between CPU and GPU

2. **Data Format/Layout Mismatch**
   - Vertex layout in shader vs actual data
   - Matrix layout (row-major vs column-major)
   - Texture coordinate or normal direction

3. **Resource Binding Issue**
   - Descriptor table might not point to correct resources
   - SRV handle might be invalid when GPU reads it
   - Uniform buffer bindings might not match shader expectations

4. **Depth Testing Configuration**
   - Depth comparison function might be inverted
   - Depth writes might be disabled
   - Geometry might be behind the camera or clipped

## Test Commands

### Native Linux (Vulkan - Working)
```bash
./target/release/rusty_renderer --backend vulkan --scene scenes/gltf_textured.toml --headless --max-frames 1 --screenshot vk_test.png
```

### Proton (DirectX - Broken)
```bash
./run_with_proton.sh --headless --max-frames 1 --screenshot dx_test.png
```

### Enable Validation
```bash
./run_with_proton.sh --headless --max-frames 1 --screenshot dx_test.png --validation
```

## Files to Investigate
- `src/backends/directx/dx12_impl.rs` - Main implementation
  - Lines 1087-1266: `capture_frame()`
  - Lines ~3000+: `DirectXPassContext` implementation
  - Drawing and command submission code
- `src/passes/forward_simple.rs` - Forward rendering pass
- `src/backends/directx/mod.rs` - Backend interface

## Comparison with Vulkan
Vulkan successfully renders the same scene, so the issue is DirectX-specific:
- Scene data is correct (works in VK)
- Camera matrices are correct (both use same calculation)
- Texture data is valid (same file loaded)
- Shader logic should be equivalent (HLSL for both backends)

## Current Hypothesis

Based on image analysis:
- DX output contains ONLY the clear color [0.1, 0.1, 0.2] (max pixel values: 26, 26, 51 out of 255)
- VK output contains geometry (max pixel values: 255, 241, 213)
- **Geometry is NOT being rasterized in DirectX**

Possible causes:
1. **Vertices transformed to clip space incorrectly** - All vertices outside view frustum
2. **Rasterizer state issue** - Triangles being culled incorrectly
3. **Shader not outputting to SV_POSITION correctly** - GPU doesn't know where to draw
4. **Root signature mismatch** - Shader can't access push constants, crashes/exits early
5. **Constant buffer binding issue** - Push constants contain garbage, transforms are invalid

## Root Cause Analysis

The geometry is not being rasterized. All rendering pipeline components work (clear, buffers bound, draw called), but pixels never reach the framebuffer beyond the clear color.

**Most Likely Issue: Coordinate System Mismatch**

1. Camera matrices use right-handed coordinate system (`Mat4::perspective_rh`, `Mat4::look_at_rh`)
2. DirectX natively uses left-handed coordinates
3. Changed `FrontCounterClockwise` from TRUE to FALSE (may have made it worse)
4. Matrices might be transforming geometry completely outside the view frustum

**Alternative Issues to Check:**
- Matrix layout (row-major vs column-major) in shader vs CPU
- HLSL `mul(matrix, vector)` order - should it be `mul(vector, matrix)`?
- Depth test comparison function may need reversal
- Viewport/scissor rect might be incorrect

## Recommended Fix Steps

1. **Revert FrontCounterClockwise to TRUE** (original setting)
2. **Add Y-flip to DirectX projection matrix** or use `perspective_lh` instead of `perspective_rh`
3. **Test with simple constant color shader** to verify PS runs
4. **Log transformed vertex positions** to see if they're in NDC range [-1,1]
5. **Try disabling depth testing** to rule out depth issues
6. **Check matrix order in HLSL** - swap mul() operands if needed

## Files to Modify
- `src/backends/directx/dx12_impl.rs:1827` - Rasterizer state (FrontCounterClockwise)
- `src/camera/mod.rs:56-59` - DirectX projection matrix
- `shaders/hlsl/forward_simple.hlsl:101` - Vertex shader transform order

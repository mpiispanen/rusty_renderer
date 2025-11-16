# DirectX Backend Progress - 2025-11-16

## Issues Fixed

### 1. Node Mask Error (E_INVALIDARG - 0x80070057)
**Problem**: CreationNodeMask and VisibleNodeMask were set to 0 in D3D12_HEAP_PROPERTIES structures, causing "Invalid parameter" errors during resource creation.

**Solution**: Changed all occurrences of CreationNodeMask and VisibleNodeMask from 0 to 1 in dx12_impl.rs. According to DirectX documentation, these values must be set to 1 for single-GPU systems.

**Files Changed**:
- `src/backends/directx/dx12_impl.rs`: Fixed 4 occurrences of node mask values

### 2. Command Line Argument Error
**Problem**: The run_with_proton.sh script was using `--output` flag, but the actual flag is `--screenshot`.

**Solution**: Updated examples in run_with_proton.sh to use correct `--screenshot` flag.

**Files Changed**:
- `run_with_proton.sh`: Updated example usage

## Current Status

### Working:
- ✅ Vulkan backend renders correctly in headless mode
- ✅ Device initialization succeeds with Proton
- ✅ Resource creation (textures, buffers) works
- ✅ Node mask issue resolved
- ✅ Pipeline creation succeeds ("Pipeline created successfully")
- ✅ Scene loading works
- ✅ Texture uploads complete

### Issues Remaining:
- ❌ E_INVALIDARG (0x80070057) in `execute_graph` after resource allocation
  - Error occurs after "Resource allocation complete: 4 textures, 4 buffers"
  - Stack trace shows error in `DirectXBackendImpl::execute_graph`
  - All initialization steps complete successfully
  - Issue is likely in command list recording or pipeline state setting

## Next Steps

1. **Add detailed logging in execute_graph** to identify exactly which DirectX API call is failing
   - Log before/after each major operation (SetPipelineState, SetGraphicsRootSignature, etc.)
   - Check viewport/scissor rect setup
   - Verify resource state transitions
   
2. **Check command list recording**
   - Verify all required root parameters are set
   - Check that descriptor heaps are set before drawing
   - Ensure proper synchronization

3. **Verify pipeline/root signature compatibility**
   - Root signature might not match shader expectations
   - Check descriptor table ranges
   - Verify root parameter indices

4. **Test with simplified rendering**
   - Try a simple clear-only path first
   - Add draw commands incrementally

## Test Commands

```bash
# Vulkan (working)
cargo run --release -- --backend vulkan --headless --max-frames 1 --screenshot vk_test.png

# DirectX with Proton (shader compilation fails)
./run_with_proton.sh --headless --max-frames 1 --screenshot dx_test.png
```

## Shader Model

Currently targeting SM 6.0 for maximum compatibility:
- WARP supports up to SM 6.2
- vkd3d-proton supports up to SM 6.8
- Using SM 6.0 for broadest compatibility

## Build Configuration

Shaders are compiled using DXC with:
- `-T vs_6_0` / `-T ps_6_0`
- `-E VSMain` / `-E PSMain`
- `-Qstrip_reflect` (strip reflection data)

DXIL output files:
- `shaders/forward_simple.vert.dxil`
- `shaders/forward_simple.frag.dxil`
- `shaders/shadow_map.vert.dxil`
- `shaders/shadow_map.frag.dxil`

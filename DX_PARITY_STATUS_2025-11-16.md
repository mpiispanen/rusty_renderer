# DirectX Backend Parity Status - 2025-11-16

## Summary
The DirectX backend is now successfully compiling and rendering with Proton. Both backends are operational, but there are rendering differences that need to be investigated.

## Recent Fixes

### 1. Removed Hardcoded Pipeline Creation
**Problem**: The DX backend was calling an old `create_pipeline()` function during initialization that tried to compile legacy `forward.hlsl` shaders with SM 5.0, which caused syntax errors.

**Solution**: Removed calls to `create_pipeline()` from `initialize_headless()` and `initialize_windowed()` in `dx12_impl.rs`. Pipeline creation now happens exclusively through render graph compilation, which uses the modern `forward_simple.hlsl` shader with SM 6.0.

### 2. Successfully Rendering with Proton
- DX backend now initializes without errors
- Shaders compile successfully (SM 6.0)
- Render graph executes and draws geometry (36 indices for textured cube)
- Screenshots are captured successfully

## Current Status

### ✅ Working
- DX backend initialization (headless mode)
- Shader compilation (DXIL SM 6.0)
- Render graph compilation and execution
- Resource allocation (buffers, textures)
- Descriptor table binding
- Draw calls (DrawIndexedInstanced)
- Screenshot capture via Proton
- Vulkan backend (fully functional)

### ⚠️ Issues to Investigate

#### 1. Rendering Differences Between Backends
- **Observation**: VK and DX screenshots have completely different pixels (921,600 out of 921,600 differ)
- **File sizes**: VK = 58K, DX = 51K
- **Both render the damaged helmet scene**, but visual output differs
- **Next steps**: 
  - Check if coordinate system differences are causing flipping
  - Verify clear color is the same
  - Check texture sampling
  - Verify depth testing

#### 2. Clear Color Mismatch
- Clear color should be defined in render pass, not hardcoded in backends
- Need to ensure both backends use the same clear color from render pass definition

#### 3. Windowed Mode Testing
- DX windowed mode not yet tested with latest changes
- Need to verify blit to swapchain works correctly

## Build Information
- **Shader Model**: 6.0 (SM 6.0) for both VS and PS
- **Compilation**: DXC for DXIL (DX) and SPIR-V (Vulkan)
- **Cross-compilation**: x86_64-pc-windows-gnu target
- **Proton**: 9.0 Beta
- **VKD3D**: Latest version with SM 6.6-6.8 support

## Testing Commands

### DX with Proton (Headless)
```bash
./run_with_proton.sh --headless --max-frames 1 --screenshot dx_test.png
```

### Vulkan (Headless)
```bash
./target/release/rusty_renderer --backend vulkan --headless --max-frames 1 --screenshot vk_test.png
```

### Compare Screenshots
```bash
compare -metric AE vk_test.png windows_test_directx/dx_test.png diff.png
```

## Next Steps

1. **Debug Rendering Differences**
   - Add detailed logging for shader inputs
   - Verify projection matrix is correct for both backends
   - Check texture coordinates and sampling
   - Verify depth buffer state

2. **Coordinate System Fix**
   - Earlier observations showed helmet was upside-down in DX
   - Need to verify if Y-axis flip is properly handled
   - Check if viewport or projection matrix needs adjustment

3. **CI Integration**
   - Update CI to test both backends
   - Add WARP testing for Windows
   - Keep Proton testing on GPU node
   - Set up image comparison in CI

4. **Render Pass Architecture**
   - Move hardcoded bindings to render pass definitions
   - Remove special cases for depth/shadow in application code
   - Ensure all rendering logic is in render graph/passes

## Files Changed
- `src/backends/directx/dx12_impl.rs`: Removed hardcoded pipeline creation
- Render graph now handles all pipeline compilation

## Performance Notes
- DX backend executes successfully without GPU faults
- No "pending command lists" errors after synchronization fixes
- Exit code 0 for all tests

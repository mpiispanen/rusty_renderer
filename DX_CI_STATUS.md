# DirectX CI Status

## Current Issue

Both Windows WARP and Linux Proton DirectX tests are failing with the same error:

```
Error: Failed to initialize headless mode

Caused by:
    The parameter is incorrect. (0x80070057)
```

## Root Cause

The error occurs during `CreateGraphicsPipelineState` in the `create_pipeline()` function. The D3D12 pipeline state descriptor has an invalid parameter.

## Evidence

1. **Windows WARP** (native D3D12): Fails with 0x80070057
2. **Linux Proton** (VKD3D translation): Fails with same error
3. **Vulkan**: Works correctly

The fact that both native Windows and Proton fail identically proves this is not a VKD3D compatibility issue but a bug in our DirectX backend code.

## Recent Changes

- Changed shader model from 6.0 to 5.1 for better Proton compatibility
- Shaders compile successfully at build time
- Shaders load correctly at runtime
- Error occurs when creating pipeline state object (PSO)

## Next Steps

1. Review D3D12_GRAPHICS_PIPELINE_STATE_DESC parameters
2. Verify shader bytecode is valid DXIL
3. Check root signature compatibility with shader model 5.1
4. Add validation layer output to CI for better error messages
5. Consider testing with simpler shaders first

## Workaround

For now, CI tests Vulkan backend which is working correctly. DirectX tests are expected to fail until the pipeline creation issue is resolved.

## Files to Investigate

- `src/backends/directx/dx12_impl.rs`: Lines 536-818 (`create_pipeline` function)
- `build.rs`: DXIL shader compilation
- `shaders/hlsl/forward_simple.hlsl`: Shader source


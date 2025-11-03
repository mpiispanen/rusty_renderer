# DirectX VKD3D-Proton Shader Compilation Issue

## Status
DirectX backend shader compilation is failing when running through VKD3D-Proton on Linux.

## Error
```
vkd3d-proton:vkd3d_compile_shader_stage: Failed to compile shader, vkd3d result -3.
```

VKD3D error code -3 corresponds to `VKD3D_ERROR_INVALID_SHADER`.

## Investigation
1. DXIL bytecode is generated correctly with DXC (shader model 6.0)
2. Files have correct DXBC headers and sizes
3. Tried multiple DXC compilation flags:
   - `-Vd` (disable validation)
   - `-validator-version 0.0` (use internal validator)
4. PSO descriptor is correctly formed
5. Bytecode is loaded and passed correctly to CreateGraphicsPipelineState

## Root Cause
VKD3D-Proton's DXIL-to-SPIR-V compiler is rejecting our shader bytecode. This could be due to:
- Incompatibility between our DXC version and VKD3D-Proton
- Missing DXIL features or flags
- VKD3D-Proton bugs

## Workaround
None currently available. DirectX backend should be tested on native Windows.

## TODO
1. Test DirectX backend on native Windows (no VKD3D translation layer)
2. Try older/newer VKD3D-Proton versions
3. Consider using FXC instead of DXC for SM 5.1 shaders (broader compatibility)
4. Report issue to VKD3D-Proton if confirmed to be a bug

## Notes
- Vulkan backend works correctly with same HLSL source compiled to SPIR-V
- This only affects Linux testing through Proton/Wine
- Real Windows DirectX should work fine

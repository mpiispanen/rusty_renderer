# DirectX Backend Proton Compatibility Issues

## Current Status
The DirectX backend compiles and initializes successfully but fails when creating the graphics pipeline state under vkd3d-proton (Proton/Wine).

## Error Details

### vkd3d-proton Error
```
warn:vkd3d-proton:vkd3d_compile_shader_stage: Failed to compile shader, vkd3d result -3.
```

### Application Error
```
ERROR: Failed to initialize headless mode
Caused by: Invalid parameter. (0x80070057)
```

The error occurs during `CreateGraphicsPipelineState()` after the root signature has been successfully created.

## Root Cause Analysis

### vkd3d Error Code -3
Error code `-3` in vkd3d typically indicates `VKD3D_ERROR_INVALID_SHADER_BYTECODE`, meaning:
- The DXIL bytecode is malformed, OR
- The DXIL bytecode contains features/patterns that vkd3d-proton's DXIL-to-SPIR-V translator doesn't support

### What We've Tried

1. **Different Shader Models**
   - Tried SM 6.0 (original)
   - Tried SM 5.1 
   - DXC automatically promotes 5.1 to 6.0 anyway
   - No change in behavior

2. **Shader Features Analysis**
   Our `forward_simple.hlsl` uses:
   - Root constants (48 DWORDs for matrices)
   - Multiple CBVs (b0, b1, b2)
   - Texture sampling (Texture2D + SamplerState)
   - **SamplerComparisonState for shadow mapping** ← Potential issue
   - Dynamic loops
   - Complex matrix operations

3. **Root Signature**
   - Root signature creates successfully
   - Contains: 2 CBVs, 32-bit constants, 1 descriptor table, 2 static samplers
   - This suggests the root signature itself is valid

## Potential Solutions

### 1. Simplify Shader Features
Remove advanced features that may not translate well:
- Remove `SamplerComparisonState` and shadow mapping
- Simplify or remove dynamic loops  
- Reduce root constant size
- Test with a minimal vertex+pixel shader

### 2. Use SPIR-V Directly
Instead of DXIL that vkd3d-proton must translate:
- Compile HLSL to SPIR-V using DXC (`-spirv` flag)
- Load SPIR-V blobs directly when running under Proton
- This bypasses vkd3d-proton's DXIL->SPIR-V translation

### 3. Test on Native Windows
- Build and test on actual Windows with real D3D12
- This would tell us if the DXIL is valid or if it's a vkd3d-proton-specific issue

### 4. Use Simpler Pipeline State
- Check if any PSO descriptor fields are causing issues
- Verify all descriptor formats and flags are compatible

## Known vkd3d-proton Limitations

From vkd3d-proton documentation and known issues:
1. Not all DXIL features are supported
2. Some shader model 6.x features may not translate correctly
3. Complex root signatures can cause issues
4. Shadow mapping with comparison samplers has known issues in some cases

## Recommended Next Steps

1. Create a minimal test shader without:
   - Shadow mapping / SamplerComparisonState
   - Complex lighting calculations
   - Large root constant blocks

2. If minimal shader works, incrementally add features back to identify the problematic one

3. Consider switching to SPIR-V compilation path for Linux/Proton builds

4. Test on actual Windows hardware if available to rule out shader validity issues

## Related Files

- `src/backends/directx/dx12_impl.rs` - Pipeline creation code
- `shaders/hlsl/forward_simple.hlsl` - Current shader
- `build.rs` - Shader compilation configuration
- `run_with_proton.sh` - Proton test script

## Timeline

- Initial DX backend implementation: Working on Windows
- Proton testing: Shader compilation failure discovered
- Attempted fixes: SM version changes, shader simplification - no success yet

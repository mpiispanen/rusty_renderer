# DirectX/Vulkan Parity Status - 2025-11-02

## Current Status

### Working ✅
- **Vulkan Backend**: Fully functional
  - Camera backend switching implemented
  - Y-flip for DirectX projection matrix
  - Cube renders correctly with vertex colors and lighting
  - Screenshot: `cube_vulkan.png`

### In Progress ⚠️
- **DirectX Backend**: PSO Creation Failing
  - Camera backend switching implemented
  - Shader compilation to DXIL succeeds
  - vkd3d-proton fails to compile shader (error -3)
  - Error: "Invalid parameter" (0x80070057) when creating PSO

## Root Cause Analysis

The DirectX shader is failing vkd3d-proton validation. The issue is with how push constants are declared:

### Current Shader Structure
```hlsl
#ifdef VULKAN
[[vk::push_constant]] PushConstantData pushConstants;
#else
// DirectX uses root constants at b2
cbuffer PushConstants : register(b2) {
    PushConstantData pushConstants;
};
#endif
```

### Root Signature Setup (DirectX)
```rust
// Root parameter 2: Push constants (32 DWORDs = 128 bytes)
D3D12_ROOT_PARAMETER {
    ParameterType: D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS,
    Constants: D3D12_ROOT_CONSTANTS {
        ShaderRegister: 2, // b2 in HLSL
        RegisterSpace: 0,
        Num32BitValues: 32, // 128 bytes / 4 = 32 DWORDs
    },
    ShaderVisibility: D3D12_SHADER_VISIBILITY_VERTEX,
}
```

## Problem

The shader declares a `cbuffer` at b2, but the root signature expects **root constants** (not a cbuffer).

In DirectX 12:
- **Root Constants** = inline constants passed via `SetGraphicsRoot32BitConstants()`
- **CBV (Constant Buffer View)** = pointer to a GPU buffer

These are different! Our root signature uses `ROOT_PARAMETER_TYPE_32BIT_CONSTANTS`, but the shader declares a `cbuffer`.

## Solution

We need to remove the cbuffer wrapper for DirectX and access the push constants directly. However, HLSL doesn't have a direct equivalent to Vulkan's push constants. Options:

### Option 1: Use Root Constants Properly
HLSL doesn't have syntax for root constants directly in SM 6.0. We need to either:
1. Keep as cbuffer and change root signature to use CBV
2. Or find a way to make DXC generate the right metadata

### Option 2: Switch to CBV for "Push Constants"
Change the DirectX backend to use a small constant buffer instead of root constants.

### Option 3: Use Specialization
Use a small staging buffer that's updated per-draw and bound as a CBV.

## Recommended Next Steps

1. **Try changing root signature to use CBV instead of root constants**
   - Change `D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS` → `D3D12_ROOT_PARAMETER_TYPE_CBV`
   - Keep shader as-is with `cbuffer`
   - Update `push_constants()` to create/update a small GPU buffer

2. **Alternative: Investigate DXC flags**
   - Check if there's a way to compile root constants properly with DXC

3. **Test with simpler shader**
   - Try a shader that doesn't use push constants at all
   - Verify PSO creation works

4. **Compare with triangle shader**
   - The triangle shader might have the same issue
   - Check if it works

## Files to Modify

- `src/backends/directx/dx12_impl.rs` - Root signature and PSO creation
- `shaders/hlsl/forward_simple.hlsl` - Shader declarations
- `build.rs` - Shader compilation flags

## Reference

- DirectX root signature docs: https://docs.microsoft.com/en-us/windows/win32/direct3d12/root-signatures
- DXC compiler flags: https://github.com/microsoft/DirectXShaderCompiler/wiki

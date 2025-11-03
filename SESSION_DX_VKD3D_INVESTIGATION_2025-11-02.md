# DirectX VKD3D Investigation Session - 2025-11-02

## Goal
Bring DirectX backend to parity with Vulkan backend for rendering.

## Issues Encountered

### 1. DirectX PSO Creation - Initial Error
**Error**: `Invalid parameter. (0x80070057)`  
**Cause**: PSO descriptor used `..Default::default()` which might not properly initialize all fields  
**Fix**: Explicitly initialized all PSO descriptor fields including:
- DS, HS, GS shader stages (set to default/empty)
- StreamOutput descriptor
- All other fields

### 2. DirectX VKD3D Shader Compilation Failure  
**Error**: `vkd3d-proton:vkd3d_compile_shader_stage: Failed to compile shader, vkd3d result -3`  
**VKD3D Error Code**: -3 = `VKD3D_ERROR_INVALID_SHADER`

**Investigation Steps**:
1. Verified DXIL bytecode has correct DXBC header
2. Verified file sizes match (5632 bytes for VS, 5060 bytes for PS)
3. Tried multiple DXC compilation flags:
   - `-Vd` (disable validation)
   - `-validator-version 0.0` (use internal validator)
4. Disabled depth/stencil testing
5. Removed input layout
6. All attempts failed with same error

**Root Cause**: VKD3D-Proton's DXIL-to-SPIR-V compiler is rejecting our shader bytecode. This is a known limitation of VKD3D-Proton which can be incompatible with certain DXIL bytecode, especially from newer DXC versions.

**Status**: **BLOCKED** - Cannot test DirectX through VKD3D-Proton

### 3. Vulkan Backend Hang
**Symptom**: Application hangs when running Vulkan backend after recent changes  
**Status**: Needs investigation

## Recommendations

1. **DirectX Testing**: Test DirectX backend on native Windows (without VKD3D translation layer)
   - Cross-compile with: `cargo build --release --target x86_64-pc-windows-gnu`
   - Copy binary and assets to Windows machine
   - Expected to work correctly on native DirectX 12

2. **VKD3D Alternatives**:
   - Try older VKD3D-Proton versions
   - Consider using FXC compiler for SM 5.1 shaders (better compatibility)
   - Report issue to VKD3D-Proton project if confirmed bug

3. **Vulkan Focus**: Fix Vulkan backend hang and ensure it's working correctly first

## Files Modified
- `src/backends/directx/dx12_impl.rs`: PSO descriptor initialization, shader validation
- `build.rs`: DXC compilation flags for DXIL generation
- `DX_VKD3D_SHADER_ISSUE.md`: Documentation of VKD3D issue

## Next Steps
1. Fix Vulkan backend hang
2. Ensure Vulkan renders correctly (cube with proper lighting)
3. Test DirectX on native Windows when available
4. Consider fallback shader compilation strategy for VKD3D compatibility

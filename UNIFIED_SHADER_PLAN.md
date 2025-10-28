# Unified Shader Compilation Plan

## Goal
Compile HLSL shaders to SPIR-V for both Vulkan and DirectX backends, ensuring identical shader code and eliminating the 13% rendering difference.

## Current State

### Vulkan Backend
- Language: GLSL
- Files: `shaders/forward.vert`, `shaders/forward.frag`
- Compilation: GLSL → SPIR-V (via glslangValidator)
- Loading: Embedded SPIR-V bytecode

### DirectX Backend
- Language: HLSL
- File: `shaders/hlsl/forward.hlsl`
- Compilation: HLSL → DXIL (runtime via D3DCompile)
- Loading: Runtime compilation or embedded bytecode

### Problem
- Different shader languages produce different results
- Manual translation introduces bugs
- 13% RMSE difference in output

## Solution: HLSL → SPIR-V for Both Backends

### Approach
Use Microsoft's DXC compiler to compile HLSL to SPIR-V:
1. Write shaders once in HLSL
2. Compile HLSL → SPIR-V for Vulkan
3. Compile HLSL → DXIL for DirectX (or use SPIR-V via vk3d-proton)
4. Identical shader code guarantees identical output

### Tools Available

#### DXC (DirectXShaderCompiler)
- Official Microsoft compiler
- Supports: HLSL → SPIRV, DXIL
- Cross-platform (Windows, Linux, macOS)
- Command: `dxc -spirv -T vs_6_0 -E VSMain shader.hlsl -Fo output.spv`

#### glslang (Vulkan SDK)
- Can compile HLSL → SPIR-V
- Command: `glslangValidator -D -V -e main -S vert shader.hlsl -o output.spv`
- Already installed (we use it for GLSL)

## Implementation Plan

### Phase 1: Setup DXC
- [ ] Install DXC compiler
- [ ] Test HLSL → SPIR-V compilation
- [ ] Verify SPIR-V is valid (spirv-val)

### Phase 2: Update Build Script
- [ ] Add HLSL → SPIR-V compilation to build.rs
- [ ] Compile forward.hlsl to SPIR-V
- [ ] Keep HLSL → DXIL for DirectX (or use SPIR-V)

### Phase 3: Update Vulkan Backend
- [ ] Point Vulkan to HLSL-generated SPIR-V
- [ ] Test rendering
- [ ] Verify no regressions

### Phase 4: Verify DirectX
- [ ] Keep using HLSL (already working)
- [ ] Or: Test SPIR-V via vkd3d-proton
- [ ] Verify output matches Vulkan

### Phase 5: Validate Parity
- [ ] Run both backends
- [ ] Compare screenshots
- [ ] Target: <1% RMSE difference

## Technical Details

### DXC SPIRV Options
```bash
dxc -spirv \
    -T vs_6_0 \              # Shader model (vs_6_0, ps_6_0, etc.)
    -E VSMain \              # Entry point
    shader.hlsl \
    -Fo output.spv \         # Output file
    -fspv-target-env=vulkan1.2  # Target Vulkan version
```

### Shader Entry Points
Current HLSL has:
- Vertex: `VSMain`
- Fragment: `PSMain`

SPIR-V expects:
- Entry points can be anything (specified at pipeline creation)
- Will use same names: VSMain, PSMain

### Descriptor Set Mapping
HLSL uses register syntax:
```hlsl
cbuffer CameraUniforms : register(b0) { }  // Maps to set=0, binding=0
Texture2D tex : register(t0)              // Maps to set=0, binding=2
SamplerState samp : register(s0)          // Maps to set=0, binding=2
```

DXC automatically maps these to SPIR-V descriptor sets.

## Risks & Mitigation

### Risk: DXC Not Available in CI
**Mitigation:** 
- Pre-compile shaders, commit SPIR-V to repo
- Or install DXC in CI (it's available via package managers)

### Risk: SPIR-V Different from DXIL
**Mitigation:**
- Use same HLSL source for both
- Test both compilation paths
- vkd3d-proton can use SPIR-V for DirectX

### Risk: Descriptor Layout Differences
**Mitigation:**
- Use explicit binding annotations in HLSL
- Verify mappings in both backends
- Test with actual rendering

## Success Criteria

1. ✅ Single HLSL source file for all shaders
2. ✅ SPIR-V generated from HLSL for Vulkan
3. ✅ Both backends render identically
4. ✅ RMSE < 1% between backends
5. ✅ CI passes with no warnings

## Next Steps

1. Install DXC compiler locally
2. Test HLSL → SPIR-V compilation
3. Compare output with current GLSL-generated SPIR-V
4. Update build script
5. Test both backends

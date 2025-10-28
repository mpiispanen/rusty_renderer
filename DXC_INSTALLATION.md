# DXC Installation Complete ✅

**Date:** 2025-10-28

## Installation

DXC (DirectX Shader Compiler) has been installed from the official Microsoft release.

### Location
- **Binaries:** `~/.local/dxc/bin/`
- **Libraries:** `~/.local/dxc/lib/`
- **Includes:** `~/.local/dxc/include/`
- **Symlinks:** `~/.local/bin/dxc` → `~/.local/dxc/bin/dxc`

### Environment
Added to `~/.bashrc`:
```bash
export PATH="$HOME/.local/bin:$PATH"
export LD_LIBRARY_PATH="$HOME/.local/dxc/lib:$LD_LIBRARY_PATH"
```

### Version
```
libdxcompiler.so: 1.9(dev;4950-b106a961)
libdxil.so: 1.9
```

## Usage

### Compile HLSL to SPIR-V for Vulkan
```bash
dxc -spirv \
    -T vs_6_0 \
    -E VSMain \
    -fspv-target-env=vulkan1.2 \
    shader.hlsl \
    -Fo output.vert.spv
```

### Compile HLSL to DXIL for DirectX
```bash
dxc -T vs_6_0 \
    -E VSMain \
    shader.hlsl \
    -Fo output.dxil
```

## DXC vs glslang

### DXC Advantages
- ✅ Official Microsoft compiler
- ✅ Latest HLSL features (SM 6.x)
- ✅ Better optimization
- ✅ More accurate HLSL semantics

### glslang Advantages (Current Choice)
- ✅ Already integrated
- ✅ Simpler `#ifdef VULKAN` syntax
- ✅ Cbuffer syntax works as-is
- ✅ No refactoring needed

## Push Constants Syntax Difference

### glslang (Current)
```hlsl
#ifdef VULKAN
[[vk::push_constant]]
#endif
cbuffer PushConstants
#ifndef VULKAN
: register(b2)
#endif
{
    float4x4 model;
};
```

### DXC (Alternative)
```hlsl
struct PushConstantsType {
    float4x4 model;
};

[[vk::push_constant]] PushConstantsType push;

// Usage: push.model instead of model
```

## Current Status

✅ DXC installed and working
✅ Can compile HLSL to SPIR-V
✅ Can compile HLSL to DXIL  
⏸️ **Keeping glslang for now** - already works well

## When to Switch to DXC

Consider switching when:
1. Need SM 6.0+ features (ray tracing, mesh shaders)
2. Need better optimization
3. Hit glslang limitations
4. Want to use DXIL for DirectX instead of runtime compilation

## Testing

Test DXC with our forward shader:
```bash
cd /var/home/matpii01/rusty_renderer

# Vertex shader
dxc -spirv -T vs_6_0 -E VSMain -fspv-target-env=vulkan1.2 \
    shaders/hlsl/forward.hlsl -Fo test_dxc.vert.spv

# Fragment shader  
dxc -spirv -T ps_6_0 -E PSMain -fspv-target-env=vulkan1.2 \
    shaders/hlsl/forward.hlsl -Fo test_dxc.frag.spv

# Validate
spirv-val test_dxc.vert.spv
spirv-val test_dxc.frag.spv
```

**Note:** Current forward.hlsl needs syntax adjustment for DXC (use struct for push constants).

## Recommendation

**Continue using glslang** for now:
- Already integrated and working
- Simpler conditional syntax
- No refactoring needed
- Can switch to DXC anytime in the future

DXC is ready and available when needed! 🚀

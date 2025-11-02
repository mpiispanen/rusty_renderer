# Unified HLSL Shader System

## Overview

As of November 2, 2025, rusty_renderer uses a unified shader system where all shaders are written in HLSL and compiled to the appropriate format for each backend:
- **Vulkan**: HLSL → SPIR-V (using DXC)
- **DirectX 12**: HLSL → DXIL (using DXC)

This ensures both backends use identical shader logic, eliminating rendering discrepancies caused by shader differences.

## Shader Compilation

### Build-Time Compilation

Shaders are compiled during the build process in `build.rs`:

```bash
# Vulkan (SPIR-V)
dxc -spirv -T vs_6_0 -E VSMain -fspv-target-env=vulkan1.2 \
    -Fo shaders/forward_simple.vert.spv shaders/hlsl/forward_simple.hlsl

# DirectX (DXIL)
dxc -T vs_6_0 -E VSMain \
    -Fo forward_simple_vs.cso shaders/hlsl/forward_simple.hlsl
```

### Prerequisites

**DXC (DirectX Shader Compiler)** is required for shader compilation. It's available for all platforms:

#### Linux Installation
```bash
# Download from: https://github.com/microsoft/DirectXShaderCompiler/releases
# Extract and install to ~/.local
mkdir -p ~/.local/bin ~/.local/lib
cp bin/dxc ~/.local/bin/
cp lib/*.so ~/.local/lib/
chmod +x ~/.local/bin/dxc

# Add to PATH and LD_LIBRARY_PATH in ~/.bashrc:
export PATH="$HOME/.local/bin:$PATH"
export LD_LIBRARY_PATH="$HOME/.local/lib:$LD_LIBRARY_PATH"
```

#### Windows
DXC is included with the Windows SDK or can be downloaded separately.

#### macOS
```bash
brew install directx-shader-compiler
```

## Shader Structure

All shaders follow HLSL syntax with Vulkan-compatible features:

```hlsl
// Camera uniforms (set 0, binding 0)
cbuffer CameraUniforms : register(b0) {
    float4x4 viewProj;
};

// Push constants (Vulkan) / Root constants (DirectX)
struct PushConstantData {
    float4x4 model;
    float4x4 normalMatrix;
};
[[vk::push_constant]] PushConstantData pushConstants;

// Vertex shader
PSInput VSMain(VSInput input) {
    // Shader code...
}

// Pixel/Fragment shader
float4 PSMain(PSInput input) : SV_TARGET {
    // Shader code...
}
```

## Backend Differences

### Coordinate Systems
- **Vulkan**: Y-axis inverted in NDC, handled via `-fvk-use-gl-layout` DXC flag
- **DirectX**: Standard DirectX coordinate system

### Resource Binding
- **Vulkan**: Uses descriptor sets and push constants
- **DirectX**: Uses root signatures and root constants

The `[[vk::push_constant]]` attribute ensures push constants work on both backends.

## Adding New Shaders

1. **Create HLSL source** in `shaders/hlsl/your_shader.hlsl`
2. **Add to build.rs**:
```rust
let shaders = vec![
    // Existing shaders...
    ("shaders/hlsl/your_shader.hlsl", "your_shader", "VSMain", "PSMain"),
];
```
3. **Register in render pass**:
```rust
pub fn register_shaders(graph: &mut RenderGraph) {
    graph.register_shader(
        "your_shader.vert",
        ShaderDescriptor::from_compiled("shaders/your_shader.vert.spv", ShaderStage::Vertex)
            .with_entry_point("VSMain"),
    );
    graph.register_shader(
        "your_shader.frag",
        ShaderDescriptor::from_compiled("shaders/your_shader.frag.spv", ShaderStage::Fragment)
            .with_entry_point("PSMain"),
    );
}
```

## Current Shaders

### forward_simple.hlsl
- **Purpose**: Simplified forward rendering with vertex colors and basic lighting
- **Features**: 
  - Multiple light sources (directional and point)
  - Ambient lighting
  - Per-vertex colors
  - No textures (simplified)
- **Compiled to**:
  - `shaders/forward_simple.vert.spv` (Vulkan)
  - `shaders/forward_simple.frag.spv` (Vulkan)
  - `OUT_DIR/forward_simple_vs.cso` (DirectX)
  - `OUT_DIR/forward_simple_ps.cso` (DirectX)

### triangle.hlsl
- **Purpose**: Simple triangle rendering for testing
- **Features**: Basic vertex transformation
- **Compiled to**:
  - `shaders/triangle.vert.spv` (Vulkan)
  - `shaders/triangle.frag.spv` (Vulkan)
  - `OUT_DIR/triangle_vs.cso` (DirectX)
  - `OUT_DIR/triangle_ps.cso` (DirectX)

## Verification

To verify both backends produce identical output:

```bash
# Vulkan
cargo run --release -- --backend vulkan --scene cube --headless --max-frames 1 --screenshot test_vk.png

# DirectX (cross-compile + Proton)
cargo build --release --target x86_64-pc-windows-gnu
./run_with_proton.sh --headless --max-frames 1 --scene cube --screenshot test_dx.png

# Compare (should be identical or very similar)
compare test_vk.png windows_test_directx/test_dx.png diff.png
```

## Troubleshooting

### DXC not found during build
- Ensure DXC is in your PATH
- Check with `dxc --version`
- Pre-compiled shaders will be used as fallback

### SPIR-V validation errors
- Run `spirv-val shaders/*.spv` to check validity
- Ensure DXC version is recent (1.7+)

### Rendering differences between backends
- Check shader compilation logs during build
- Verify both backends use the same `.hlsl` source
- Compare SPIR-V and DXIL intermediate representations

## Future Improvements

- [ ] Hot shader reloading in debug builds
- [ ] Shader include support for common functions
- [ ] Shader variant system (different defines)
- [ ] Automated shader validation in CI
- [ ] Shader optimization profiles (debug vs release)

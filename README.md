# Rusty Renderer

A modern, multi-backend rendering engine written in Rust with support for Vulkan, DirectX 12, and WebGPU (wgpu).

## Features

- **Multi-Backend Support**: Vulkan, DirectX 12, and wgpu backends with unified API
- **Render Graph Architecture**: Declarative render pass system with automatic resource management
- **Index Buffer Rendering**: Full support for indexed geometry on all backends
- **Cross-Platform Shader System**: HLSL source compiled to SPIR-V and DXIL
- **Scene Format**: TOML-based scene description with inline geometry and glTF support
- **Forward Rendering**: Complete forward rendering pipeline with lighting

## Quick Start

### Build and Run

```bash
# Build (Linux/macOS)
cargo build --release

# Build for Windows (cross-compilation on Linux)
cargo build --release --target x86_64-pc-windows-msvc

# Run with Vulkan
./target/release/rusty_renderer --backend vulkan --scene scenes/cube.toml

# Run with DirectX 12 (Windows or Linux with Proton)
./target/release/rusty_renderer --backend directx --scene scenes/cube.toml --headless
```

### Headless Rendering

```bash
# Render to PNG
./target/release/rusty_renderer --backend vulkan --scene scenes/cube.toml --headless --screenshot output.png --max-frames 1
```

## Architecture

The renderer uses a declarative render graph system where passes declare their requirements:

```rust
impl RenderPass for ForwardSimplePass {
    fn declare_resources(&self, graph: &mut RenderGraph) {
        // Declare buffers, textures, etc.
    }
    
    fn declare_pipeline(&self, builder: &mut PipelineBuilder) {
        builder
            .vertex_shader("forward_simple.vert")
            .fragment_shader("forward_simple.frag")
            .vertex_layout(self.vertex_layout.clone())
            .depth_test(true);
    }
    
    fn execute(&self, context: &mut dyn PassExecutionContext) {
        // Rendering commands
    }
}
```

## Scene Format

Scenes are defined in TOML format:

```toml
[metadata]
name = "My Scene"

[camera]
type = "perspective"
position = [0.0, 0.0, 5.0]
target = [0.0, 0.0, 0.0]
fov = 60.0

[[objects]]
type = "mesh"
[objects.geometry]
source = "inline"
indices = [0, 1, 2, 2, 3, 0]

[[objects.geometry.vertices]]
position = [-0.5, -0.5, 0.0]
color = [1.0, 0.0, 0.0]
```

## Development

### Requirements

- Rust 1.70+
- Vulkan SDK (for Vulkan backend)
- DXC shader compiler (for HLSL compilation)
- For Windows cross-compilation: `x86_64-pc-windows-msvc` target

### Shader Compilation

Shaders are automatically compiled during build from HLSL source:

```bash
# HLSL → DXIL (for DirectX)
dxc -T vs_6_0 -E VSMain -Fo forward_simple.vert.dxil forward_simple.hlsl

# HLSL → SPIR-V (for Vulkan)
dxc -T vs_6_0 -E VSMain -spirv -DVULKAN -Fo forward_simple.vert.spv forward_simple.hlsl
```

## Testing DirectX on Linux

Using Proton (Steam's Wine-based compatibility layer):

```bash
cd windows_test_directx
STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam" \
STEAM_COMPAT_DATA_PATH="$HOME/.proton_rusty_renderer" \
WINEDEBUG=-all \
"$HOME/.local/share/Steam/steamapps/common/Proton 9.0 (Beta)/proton" \
run rusty_renderer.exe --backend directx --scene scenes/cube.toml --headless --screenshot test.png --max-frames 1
```

## Project Status

- ✅ Multi-backend abstraction (Vulkan, DirectX 12, wgpu)
- ✅ Render graph system with declarative API
- ✅ Index buffer rendering
- ✅ Forward rendering with lighting
- ✅ glTF model loading with textures
- ✅ Cross-platform shader system (HLSL)
- 🚧 Shadow mapping (planned)
- 🚧 ImGui integration (planned)

## License

MIT

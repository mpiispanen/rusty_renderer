# Quick Start Guide - rusty_renderer

## TL;DR - Just Want to See It Work?

```bash
# Clone and build
git clone <repo>
cd rusty_renderer
cargo build --release

# Run with Vulkan (Linux)
cargo run --release --example gltf_viewer -- vulkan scenes/gltf_textured.toml

# View output
eog gltf_textured_vulkan.png
```

## Testing Different Backends

### Vulkan (Linux)
```bash
cargo run --release --example gltf_viewer -- vulkan scenes/gltf_textured.toml
```

### DirectX 12 (via Proton on Linux)
```bash
# Build Windows binary
cargo build --release --target x86_64-pc-windows-msvc --example gltf_viewer

# Run with Proton
./test_dx_proton.sh
```

### Compare Backends
```bash
./test_backends_comparison.sh
```

## Scene Files

### Available Scenes
- `scenes/gltf_textured.toml` - Textured cube with lighting (recommended)
- `scenes/textured_cube.toml` - Simple textured cube
- `scenes/triangle.toml` - Basic triangle (deprecated)

### Scene Format (TOML)
```toml
name = "My Scene"
description = "A test scene"

[[objects]]
name = "my_object"
type = "mesh"
geometry = { source = "gltf", path = "assets/models/cube.gltf" }
material = "my_material"

[[materials]]
name = "my_material"
diffuse_texture = "assets/textures/my_texture.png"

[camera]
type = "perspective"
position = [2.0, 2.0, 3.0]
target = [0.0, 0.0, 0.0]
fov = 45.0

[lighting]
ambient = [0.2, 0.2, 0.2]

[[lighting.lights]]
type = "directional"
direction = [-0.3, -1.0, -0.5]
color = [1.0, 1.0, 1.0]
intensity = 0.8
```

## Command Line Options

### gltf_viewer Example
```bash
gltf_viewer [BACKEND] <MODEL_PATH>

# Backends:
#   vulkan     - Use Vulkan backend (default)
#   wgpu       - Use wgpu backend (experimental, has issues)
#   directx    - Use DirectX 12 backend (Windows only)
#   dx12       - Alias for directx

# Examples:
gltf_viewer vulkan assets/models/cube.gltf
gltf_viewer wgpu scenes/gltf_textured.toml
gltf_viewer directx assets/models/textured_cube.gltf
```

## Build Commands

### Native (Linux/Vulkan)
```bash
cargo build --release
cargo run --release --example gltf_viewer
```

### Cross-compile to Windows (DirectX)
```bash
cargo build --release --target x86_64-pc-windows-msvc --example gltf_viewer
```

### With specific features
```bash
# Vulkan only
cargo build --release --no-default-features --features vulkan

# DirectX only (Windows)
cargo build --release --no-default-features --features directx
```

## Testing

### Run All Tests
```bash
cargo test --release
```

### Run Specific Test
```bash
cargo test --release test_name
```

### Backend Comparison
```bash
./test_backends_comparison.sh
```

## Troubleshooting

### Vulkan Validation Errors
```bash
# Disable validation layers
export VULKAN_VALIDATION=0
cargo run --example gltf_viewer -- vulkan scenes/gltf_textured.toml
```

### DirectX/Proton Issues
```bash
# Enable debug output
VKD3D_DEBUG=trace ./test_dx_proton.sh
```

### Build Errors (Windows cross-compile)
```bash
# Make sure xwin is set up
cargo install xwin
xwin --accept-license splat --output .xwin-cache
```

## Project Structure

```
rusty_renderer/
├── src/
│   ├── backends/          # Graphics backend implementations
│   │   ├── vulkan/        # Vulkan backend
│   │   ├── directx/       # DirectX 12 backend
│   │   └── wgpu_backend/  # wgpu backend (experimental)
│   ├── pipelines/         # Rendering pipelines
│   │   └── forward.rs     # Forward rendering pipeline
│   ├── passes/            # Render passes
│   │   └── forward.rs     # Forward rendering pass
│   ├── resources/         # Asset loading (GLTF, textures)
│   ├── scene/             # Scene description and loading
│   └── render_graph/      # Render graph system
├── examples/              # Example applications
│   └── gltf_viewer.rs     # Main GLTF viewer example
├── scenes/                # Test scene files (.toml)
├── assets/                # Test assets (models, textures)
└── shaders/              # Shader source files
```

## Environment Variables

### General
- `RUST_LOG` - Set logging level (info, debug, trace)
- `VULKAN_VALIDATION` - Enable/disable Vulkan validation (1/0)

### Proton/DirectX
- `VKD3D_DEBUG` - vkd3d-proton debug level (warn, info, trace)
- `STEAM_COMPAT_CLIENT_INSTALL_PATH` - Steam installation path
- `STEAM_COMPAT_DATA_PATH` - Proton prefix path

### Example
```bash
RUST_LOG=debug cargo run --example gltf_viewer -- vulkan scenes/gltf_textured.toml
```

## Performance Tips

### Debug vs Release
Always use `--release` for performance testing:
```bash
cargo run --release  # Much faster than debug builds
```

### Validation Layers
Disable in production:
```bash
export VULKAN_VALIDATION=0
```

### Profiling
```bash
cargo install cargo-flamegraph
cargo flamegraph --example gltf_viewer -- vulkan scenes/gltf_textured.toml
```

## Common Tasks

### Add a New Model
1. Place GLTF file in `assets/models/`
2. Place textures in `assets/textures/`
3. Create scene file in `scenes/` or pass GLTF path directly:
   ```bash
   cargo run --example gltf_viewer -- vulkan assets/models/my_model.gltf
   ```

### Create Custom Scene
1. Copy `scenes/gltf_textured.toml` as template
2. Modify objects, materials, camera, lighting
3. Run with your scene:
   ```bash
   cargo run --example gltf_viewer -- vulkan scenes/my_scene.toml
   ```

### Capture Output
Output is automatically saved to:
- Vulkan: `gltf_textured_vulkan.png` (or `<scene>_vulkan.png`)
- DirectX: `gltf_textured_dx12.png` (or `<scene>_dx12.png`)

## Status

### Working
- ✅ Vulkan backend (Linux)
- ✅ DirectX 12 backend (via Proton)
- ✅ GLTF loading
- ✅ Textured meshes
- ✅ Forward rendering with lighting
- ✅ Headless rendering

### Known Issues
- ⚠️ wgpu backend has bind group issues (deferred)
- ⚠️ No depth testing yet (objects render in submission order)
- ⚠️ DirectX texture uploads are placeholder

### Coming Soon
- Depth testing
- Index buffers
- Shadow mapping
- PBR materials

## Getting Help

### Documentation
- `README.md` - Project overview
- `ACHIEVEMENT_SUMMARY.md` - Current status and features
- `BACKEND_STATUS_2025-10-25_FINAL.md` - Detailed backend status
- `NEXT_STEPS_2025-10-25.md` - Future plans

### Logs
Enable detailed logging:
```bash
RUST_LOG=trace cargo run --example gltf_viewer -- vulkan scenes/gltf_textured.toml 2>&1 | tee debug.log
```

### Issues
Check existing documentation files for known issues and solutions.

---

**Happy Rendering!** 🎨🚀

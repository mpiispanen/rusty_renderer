# Quick Reference Guide

## Common Commands

### Running the Renderer

```bash
# List available scenes
cargo run -- --list-scenes

# Run with specific scene
cargo run -- --scene scenes/triangle.toml

# Run with specific backend
cargo run -- --scene scenes/cube.toml --backend vulkan
cargo run -- --scene scenes/cube.toml --backend wgpu
cargo run -- --scene scenes/cube.toml --backend directx  # Windows only

# Run with custom window size
cargo run -- --scene scenes/cube.toml --width 1920 --height 1080

# Run with frame limit (for testing)
cargo run -- --scene scenes/cube.toml --max-frames 10

# Headless mode with screenshot
cargo run -- --scene scenes/cube.toml --headless --screenshot output.png
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Check without building
cargo check
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test asset

# Run with output
cargo test -- --nocapture

# Run only library tests
cargo test --lib
```

### Development

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Update dependencies
cargo update

# Clean build artifacts
cargo clean
```

## Scene File Templates

### Basic Triangle
```toml
[metadata]
name = "My Triangle"

[[objects]]
type = "mesh"
name = "triangle"

[objects.geometry]
source = "inline"
vertices = [
    { position = [0.0, -0.5, 0.0], color = [1.0, 0.0, 0.0] },
    { position = [0.5, 0.5, 0.0], color = [0.0, 1.0, 0.0] },
    { position = [-0.5, 0.5, 0.0], color = [0.0, 0.0, 1.0] },
]

[camera]
type = "perspective"
position = [0.0, 0.0, 3.0]
target = [0.0, 0.0, 0.0]
```

### Textured Mesh
```toml
[metadata]
name = "Textured Object"

[[materials]]
name = "my_material"
base_color = [1.0, 1.0, 1.0]
diffuse_texture = "assets/textures/my_texture.png"
metallic = 0.0
roughness = 0.5

[[objects]]
type = "mesh"
name = "quad"
material = 0

[objects.geometry]
source = "inline"
vertices = [
    { position = [-0.5, -0.5, 0.0], normal = [0.0, 0.0, 1.0], uv = [0.0, 0.0] },
    { position = [ 0.5, -0.5, 0.0], normal = [0.0, 0.0, 1.0], uv = [1.0, 0.0] },
    { position = [ 0.5,  0.5, 0.0], normal = [0.0, 0.0, 1.0], uv = [1.0, 1.0] },
    { position = [-0.5, -0.5, 0.0], normal = [0.0, 0.0, 1.0], uv = [0.0, 0.0] },
    { position = [ 0.5,  0.5, 0.0], normal = [0.0, 0.0, 1.0], uv = [1.0, 1.0] },
    { position = [-0.5,  0.5, 0.0], normal = [0.0, 0.0, 1.0], uv = [0.0, 1.0] },
]

[camera]
type = "perspective"
position = [0.0, 0.0, 2.0]
target = [0.0, 0.0, 0.0]
```

### GLTF Model
```toml
[metadata]
name = "GLTF Scene"

[[objects]]
type = "gltf"
name = "imported_model"
path = "assets/models/my_model.gltf"
transform = { position = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0], scale = [1.0, 1.0, 1.0] }

[camera]
type = "perspective"
position = [2.0, 2.0, 3.0]
target = [0.0, 0.0, 0.0]
fov = 45.0
```

### With Lighting
```toml
[metadata]
name = "Lit Scene"

# ... objects and materials ...

[camera]
type = "perspective"
position = [2.0, 2.0, 3.0]
target = [0.0, 0.0, 0.0]

[lighting]
ambient = [0.2, 0.2, 0.2]

[[lighting.lights]]
type = "directional"
direction = [-0.3, -1.0, -0.5]
color = [1.0, 1.0, 1.0]
intensity = 0.8

[[lighting.lights]]
type = "point"
position = [1.5, 1.0, 2.0]
color = [1.0, 0.7, 0.3]
intensity = 1.2
```

## Asset Organization

```
rusty_renderer/
├── assets/
│   ├── textures/          # PNG, JPG, etc.
│   │   ├── test_checkerboard.png
│   │   └── my_texture.png
│   └── models/            # GLTF, GLB files
│       └── my_model.gltf
├── scenes/                # Scene TOML files
│   ├── triangle.toml
│   ├── cube.toml
│   └── my_scene.toml
└── shaders/              # GLSL shaders
    ├── simple.vert
    ├── simple.frag
    ├── forward.vert
    └── forward.frag
```

## Asset Path Formats

```toml
# Absolute path (not recommended)
diffuse_texture = "/home/user/textures/my_texture.png"

# Relative to project root (recommended)
diffuse_texture = "assets/textures/my_texture.png"

# Relative to scene file directory
diffuse_texture = "./my_texture.png"
```

## Troubleshooting

### Shader Compilation Fails
```bash
# Check if glslangValidator is installed
which glslangValidator

# Install on Linux
sudo dnf install glslang  # Fedora
sudo apt install glslang-tools  # Ubuntu
```

### Scene Won't Load
```bash
# Check scene syntax
cargo run -- --scene scenes/my_scene.toml

# List all scenes to see what's available
cargo run -- --list-scenes
```

### Texture Not Found
- Check path is correct in scene file
- Verify file exists: `ls -la assets/textures/`
- Check file permissions
- Look at logs for detailed error

### Build Errors
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

## Performance Tips

### Debug vs Release
```bash
# Debug build (slower, has debug info)
cargo run

# Release build (much faster)
cargo run --release
```

### Frame Limiting
```bash
# Limit frames for consistent benchmarking
cargo run --release -- --scene scenes/cube.toml --max-frames 1000
```

## Useful Environment Variables

```bash
# Rust logging levels
RUST_LOG=debug cargo run
RUST_LOG=info cargo run
RUST_LOG=warn cargo run

# Vulkan validation layers
VK_LAYER_PATH=/usr/share/vulkan/explicit_layer.d
```

## Documentation

- Project overview: `README.md`
- Architecture: `docs/ARCHITECTURE.md`
- Asset system: `docs/ASSETS.md`
- Roadmap: `ROADMAP.md`
- Current status: `ASSET_SYSTEM_STATUS.md`

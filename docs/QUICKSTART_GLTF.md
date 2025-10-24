# Quick Start Guide - GLTF Models

## Loading a GLTF Model

### 1. Place Your Model
```bash
# Put your GLTF/GLB file in assets/models/
cp my_model.gltf assets/models/
```

### 2. Create a Scene File
```toml
# scenes/my_scene.toml
[metadata]
name = "My Scene"
description = "My cool 3D scene"

[[objects]]
type = "gltf"
name = "my_model"
path = "assets/models/my_model.gltf"
transform = { position = [0.0, 0.0, 0.0] }

[camera]
type = "perspective"
position = [3.0, 3.0, 3.0]
target = [0.0, 0.0, 0.0]
up = [0.0, 1.0, 0.0]
fov = 45.0

[lighting]
ambient = [0.2, 0.2, 0.2]

[[lighting.lights]]
type = "directional"
direction = [-0.3, -1.0, -0.5]
color = [1.0, 1.0, 1.0]
intensity = 0.8
```

### 3. Render It
```bash
# Windowed mode (interactive)
cargo run -- --pipeline forward --scene scenes/my_scene.toml

# Headless mode (screenshot)
cargo run -- --headless --pipeline forward \
  --scene scenes/my_scene.toml \
  --screenshot output.png --max-frames 1
```

## Features

### Automatic Texture Extraction
Embedded textures are automatically extracted to `.gltf_cache/`:
```
assets/models/
  my_model.gltf
  .gltf_cache/
    my_model_mat0_basecolor.png  # Auto-extracted!
```

### Supported Features
- ✅ Meshes (indexed and non-indexed)
- ✅ Positions, normals, UVs, colors
- ✅ PBR materials (base color, metallic, roughness)
- ✅ Embedded textures (PNG, JPEG)
- ✅ Multiple primitives per mesh
- ✅ Transforms (position, rotation, scale)

### Not Yet Supported
- ⏳ External texture files
- ⏳ Normal maps
- ⏳ Metallic/roughness maps
- ⏳ Animations
- ⏳ Skeletal meshes

## Testing Your Model

```bash
# Quick validation
cargo run --example test_gltf_loader assets/models/my_model.gltf

# Full scene test
cargo run --example test_scene_gltf scenes/my_scene.toml
```

## Generating Test Models

### Simple Cube
```bash
python3 scripts/generate_gltf_cube.py
```

### Textured Cube
```bash
python3 scripts/generate_textured_gltf.py
```

## Troubleshooting

### "Failed to load texture"
- Check that `.gltf_cache/` was created
- Verify texture extraction logs
- Try deleting cache and reloading

### "Invalid GLTF file"
- Validate your GLTF at https://gltf-viewer.donmccurdy.com/
- Check that file paths are correct
- Ensure all referenced assets exist

### Model appears black
- Check that lighting is configured in scene
- Verify material has reasonable values
- Try adding a point light near the model

## Example Scenes

### Minimal
```toml
[[objects]]
type = "gltf"
path = "assets/models/cube.gltf"

[camera]
type = "perspective"
position = [2.0, 2.0, 3.0]
target = [0.0, 0.0, 0.0]
```

### With Texture
```toml
[[objects]]
type = "gltf"
path = "assets/models/textured_cube.gltf"

[camera]
type = "perspective"
position = [2.0, 2.0, 3.0]
target = [0.0, 0.0, 0.0]

[lighting]
ambient = [0.2, 0.2, 0.2]
```

## Backend Selection

```bash
# Vulkan (default, best tested)
cargo run -- --backend vulkan --pipeline forward --scene my_scene.toml

# wgpu (has some issues)
cargo run -- --backend wgpu --pipeline forward --scene my_scene.toml

# DirectX 12 (Windows only)
cargo run -- --backend directx --pipeline forward --scene my_scene.toml
```

## Tips

1. **Start Simple**: Test with the included cube models first
2. **Check Logs**: Use `RUST_LOG=info` for detailed information
3. **Validate Early**: Use test examples to check GLTF loads correctly
4. **Cache Aware**: Delete `.gltf_cache/` if textures seem wrong
5. **Scene Relative**: Paths in TOML are relative to project root

## More Information

- Full docs: `docs/ASSETS.md`
- Implementation: `GLTF_IMPLEMENTATION_COMPLETE.md`
- Session notes: `SESSION_GLTF_COMPLETE_2025-10-25.md`

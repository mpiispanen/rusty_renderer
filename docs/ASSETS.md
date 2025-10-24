# Asset Management in Rusty Renderer

This document describes how assets (textures, models, etc.) are managed and referenced in Rusty Renderer.

## Asset Path Resolution

The asset path resolver (`AssetPathResolver`) provides flexible path resolution for all assets used in scene files.

### How It Works

The resolver automatically finds the project root by looking for `Cargo.toml` and resolves paths relative to it.

### Supported Path Formats

1. **Absolute paths**: Used as-is
   ```toml
   diffuse_texture = "/absolute/path/to/texture.png"
   ```

2. **Paths starting with "assets/"**: Resolved relative to project root
   ```toml
   diffuse_texture = "assets/textures/checkerboard.png"
   # Resolves to: <project_root>/assets/textures/checkerboard.png
   ```

3. **Paths starting with "./"**: Resolved relative to scene directory
   ```toml
   diffuse_texture = "./my_texture.png"
   # If scene is in scenes/my_scene/scene.toml
   # Resolves to: scenes/my_scene/my_texture.png
   ```

4. **Other relative paths**: Resolved relative to project root
   ```toml
   diffuse_texture = "textures/test.png"
   # Resolves to: <project_root>/textures/test.png
   ```

## Asset Directory Structure

The recommended asset structure is:

```
rusty_renderer/
├── Cargo.toml
├── assets/
│   ├── textures/         # Texture files
│   │   ├── test_checkerboard.png
│   │   └── test_gradient.png
│   ├── models/           # 3D models (GLTF, GLB, etc.)
│   │   ├── cube.gltf
│   │   └── sphere.glb
│   └── materials/        # Material definitions (future)
├── scenes/               # Scene files
│   ├── triangle.toml
│   ├── textured_cube.toml
│   └── gltf_test.toml
└── src/
```

## Loading GLTF Models

GLTF models can be loaded by referencing them in scene files:

```toml
[[objects]]
type = "gltf"
name = "my_model"
path = "assets/models/cube.gltf"  # Resolved relative to project root
transform = { position = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0], scale = [1.0, 1.0, 1.0] }
```

The GLTF loader will:
- Load all meshes and primitives from the GLTF file
- Extract materials with PBR properties
- Convert vertex data to the renderer's format
- Handle embedded or referenced textures (in development)

## Using Textures in Materials

Textures are referenced in material definitions:

```toml
[[materials]]
name = "checkerboard"
base_color = [1.0, 1.0, 1.0]
diffuse_texture = "assets/textures/test_checkerboard.png"
metallic = 0.0
roughness = 0.6
```

## Path Resolution Examples

Given a project at `/home/user/rusty_renderer/`:

| Path in Scene File | Resolved To |
|-------------------|-------------|
| `assets/textures/test.png` | `/home/user/rusty_renderer/assets/textures/test.png` |
| `./texture.png` (scene in `scenes/my_scene.toml`) | `/home/user/rusty_renderer/scenes/texture.png` |
| `/tmp/texture.png` | `/tmp/texture.png` |
| `textures/test.png` | `/home/user/rusty_renderer/textures/test.png` |

## Future Improvements

- [ ] Support for asset caching and hot reloading
- [ ] Asset packaging for distribution
- [ ] Embedded texture support in GLTF
- [ ] Virtual file system for packaged assets
- [ ] Asset preprocessing and optimization

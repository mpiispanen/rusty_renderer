# Assets Directory

This directory contains runtime assets used by the Rusty Renderer.

## Structure

- `textures/` - Texture images (PNG, JPG, etc.)
- `models/` - 3D models (GLTF, GLB files)

## Test Assets

The following test assets are provided:
- `textures/test_checkerboard.png` - 8x8 checkerboard pattern (white/gray)
- `textures/test_gradient.png` - Horizontal gradient

## Adding Assets

Place your assets in the appropriate subdirectory and reference them in scene files using paths relative to the project root:

```toml
# In a scene file
[[materials]]
name = "my_material"
diffuse_texture = "assets/textures/my_texture.png"

[[objects]]
type = "gltf"
name = "my_model"
path = "assets/models/my_model.gltf"
```

See `docs/ASSETS.md` for detailed documentation on asset path resolution.

# GLTF Loading Guide

This guide demonstrates how to load and render GLTF models in Rusty Renderer.

## Quick Start

### Loading a GLTF Model Directly

```bash
cargo run --example gltf_viewer vulkan assets/models/textured_cube.gltf
```

### Loading via Scene File

```bash
cargo run --example gltf_viewer vulkan scenes/gltf_textured.toml
```

## Asset Path Resolution

The asset system automatically resolves paths relative to the project root:

```toml
# scenes/my_scene.toml
[[objects]]
type = "gltf"
path = "assets/models/my_model.gltf"  # Resolved relative to project root
```

You can also use paths relative to the scene file:

```toml
[[objects]]
type = "gltf"
path = "./models/my_model.gltf"  # Resolved relative to scene directory
```

## GLTF Feature Support

### ✅ Supported Features

- [x] Mesh primitives (triangles)
- [x] Vertex positions
- [x] Vertex normals  
- [x] Vertex UVs (texture coordinates)
- [x] Vertex colors
- [x] Index buffers
- [x] PBR materials (metallic-roughness)
- [x] Base color factor
- [x] Metallic/roughness factors
- [x] Embedded textures (extracted to cache)
- [x] Multiple meshes per model
- [x] Multiple materials

### 🚧 Planned Features

- [ ] Node transformations
- [ ] Scene hierarchy
- [ ] Animations
- [ ] Skinning
- [ ] Morph targets
- [ ] Multiple texture sets
- [ ] Normal maps
- [ ] Emissive materials

## Examples

### Example 1: Simple GLTF Model

```rust
use rusty_renderer::{
    backends::{create_backend, BackendType},
    pipelines::{ForwardPipeline, RenderPipeline},
    resources::GltfLoader,
    scene::{Camera, Lighting, Light, Scene},
};

// Load GLTF file
let (objects, materials, metadata) = GltfLoader::load("assets/models/cube.gltf")?;

// Create a scene with camera and lighting
let scene = Scene {
    metadata,
    objects,
    materials,
    camera: Camera::Perspective {
        position: [2.0, 2.0, 3.0],
        target: [0.0, 0.0, 0.0],
        up: [0.0, 1.0, 0.0],
        fov: 45.0,
        near: 0.1,
        far: 100.0,
    },
    lighting: Some(Lighting {
        ambient: [0.2, 0.2, 0.2],
        lights: vec![
            Light::Directional {
                direction: [-0.3, -1.0, -0.5],
                color: [1.0, 1.0, 1.0],
                intensity: 0.8,
            },
        ],
    }),
};

// Render
let mut backend = create_backend(BackendType::Vulkan, false)?;
backend.initialize_headless(800, 600)?;

let mut pipeline = ForwardPipeline::new();
let mut graph = pipeline.build_graph(&scene, &mut *backend)?;
let compiled = graph.compile()?;

backend.begin_frame()?;
backend.execute_graph(&graph, &compiled)?;
backend.end_frame()?;

let (width, height, pixels) = backend.capture_frame()?;
image::save_buffer("output.png", &pixels, width, height, image::ColorType::Rgba8)?;
```

### Example 2: Using Scene Files

Create a scene file that references a GLTF model:

```toml
# scenes/my_model.toml
[metadata]
name = "My Model"
description = "A GLTF model scene"
author = "Me"

[[objects]]
type = "gltf"
name = "my_model"
path = "assets/models/my_model.gltf"
transform = { position = [0.0, 0.0, 0.0], rotation = [0.0, 0.0, 0.0], scale = [1.0, 1.0, 1.0] }

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

Then load and render:

```rust
use rusty_renderer::scene::SceneLoader;

let loader = SceneLoader::new()?;
let scene = loader.load_from_file("scenes/my_model.toml")?;

// ... render as above
```

## Texture Handling

### Embedded Textures

GLTF files with embedded textures are automatically extracted to a cache directory:

```
assets/models/my_model.gltf
assets/models/.gltf_cache/
  └── my_model_mat0_basecolor.png  # Extracted automatically
```

The cache is created next to the GLTF file and is reused across runs.

### External Texture References

If your GLTF references external image files, ensure they are in the correct location relative to the GLTF file:

```
assets/models/
  ├── my_model.gltf
  └── textures/
      └── diffuse.png  # Referenced from GLTF
```

## Multi-Backend Support

The same GLTF loading code works across all backends:

```bash
# Vulkan
cargo run --example gltf_viewer vulkan assets/models/textured_cube.gltf

# wgpu (currently has issues - being debugged)
cargo run --example gltf_viewer wgpu assets/models/textured_cube.gltf

# DirectX 12 (Windows only)
cargo run --example gltf_viewer dx12 assets/models/textured_cube.gltf
```

## Performance Considerations

### Large Models

For large models with many vertices:
- Consider using indexed geometry (automatically handled)
- Monitor memory usage during loading
- Profile loading times

### Multiple Materials

Each material may require:
- Texture uploads
- Material buffer creation  
- Descriptor set binding

The system handles this efficiently, but be aware of overhead with many materials.

## Troubleshooting

### "Asset not found" Error

Check that paths are:
1. Relative to project root (for `assets/...` paths)
2. Relative to scene file (for `./...` paths)
3. Using forward slashes even on Windows

### Missing Textures

- Check `.gltf_cache` directory was created
- Verify texture files exist if externally referenced
- Look for error messages during texture loading

### Rendering Issues

- Verify normals are present (some GLTF exporters omit them)
- Check UVs are in 0-1 range
- Ensure camera is positioned correctly

## Creating Test Models

### Using Blender

1. Model your object in Blender
2. Select glTF 2.0 (.gltf/.glb) as export format
3. Enable "Apply Modifiers"
4. Enable "Include UVs" and "Normals"
5. For textures, use "Embedded" or "Separate"
6. Export to `assets/models/`

### Using Online Tools

- [glTF Sample Models](https://github.com/KhronosGroup/glTF-Sample-Models)
- [Sketchfab](https://sketchfab.com/) - Download as glTF
- [glTF Editor](https://gltf-transform.donmccurdy.com/)

## API Reference

### GltfLoader

```rust
pub struct GltfLoader;

impl GltfLoader {
    /// Load a GLTF file
    pub fn load<P: AsRef<Path>>(
        path: P,
    ) -> Result<(Vec<SceneObject>, Vec<Material>, SceneMetadata)>;
}
```

### AssetPathResolver

```rust
pub struct AssetPathResolver {
    pub fn new() -> Result<Self>;
    pub fn resolve(&self, path: &str, scene_dir: Option<&Path>) -> PathBuf;
    pub fn resolve_and_verify(&self, path: &str, scene_dir: Option<&Path>) -> Result<PathBuf>;
}
```

## See Also

- [Scene File Format](./SCENE_FORMAT.md)
- [Material System](./MATERIALS.md)
- [Asset System](./ASSETS.md)

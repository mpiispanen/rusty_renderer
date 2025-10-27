//! Scene loading from TOML files

use super::*;
use crate::resources::{AssetPathResolver, GltfLoader};
use anyhow::{Context, Result};
use std::path::Path;

/// Scene loader for loading scenes from files
pub struct SceneLoader {
    asset_resolver: AssetPathResolver,
}

impl SceneLoader {
    /// Create a new scene loader
    pub fn new() -> Result<Self> {
        Ok(Self {
            asset_resolver: AssetPathResolver::new()?,
        })
    }

    /// Create with explicit project root
    pub fn with_root<P: AsRef<Path>>(root: P) -> Self {
        Self {
            asset_resolver: AssetPathResolver::with_root(root.as_ref()),
        }
    }

    /// Load a scene from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<Scene> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scene file: {}", path.display()))?;

        let scene_dir = path.parent();
        self.load_from_string(&content, scene_dir)
    }

    /// Load a scene from a TOML file (static method for backward compatibility)
    pub fn load_from_file_static<P: AsRef<Path>>(path: P) -> Result<Scene> {
        let loader = Self::new()?;
        loader.load_from_file(path)
    }

    /// Load a scene from a TOML string
    pub fn load_from_string(&self, content: &str, scene_dir: Option<&Path>) -> Result<Scene> {
        let mut scene: Scene = toml::from_str(content).context("Failed to parse scene TOML")?;

        // Expand GLTF model references
        scene = self.expand_gltf_models(scene, scene_dir)?;

        // Resolve asset paths in materials
        for material in &mut scene.materials {
            if let Some(ref texture_path) = material.diffuse_texture {
                let resolved = self.asset_resolver.resolve(texture_path, scene_dir);
                material.diffuse_texture = Some(
                    resolved
                        .to_str()
                        .context("Invalid texture path")?
                        .to_string(),
                );
            }
        }

        // Validate the scene
        scene.validate()?;

        Ok(scene)
    }

    /// Load a scene from a TOML string (static method for backward compatibility)
    pub fn load_from_string_static(content: &str) -> Result<Scene> {
        let loader = Self::new()?;
        loader.load_from_string(content, None)
    }

    /// Expand GLTF model references into inline meshes
    fn expand_gltf_models(&self, mut scene: Scene, scene_dir: Option<&Path>) -> Result<Scene> {
        let mut expanded_objects = Vec::new();
        let mut gltf_materials_offset = scene.materials.len();

        for object in scene.objects {
            match object {
                SceneObject::GltfModel {
                    name,
                    path,
                    transform,
                } => {
                    log::info!("Loading GLTF model: {name} from {path}");

                    // Resolve path
                    let gltf_path = self
                        .asset_resolver
                        .resolve_and_verify(&path, scene_dir)
                        .with_context(|| format!("Failed to resolve GLTF path: {path}"))?;

                    // Load GLTF
                    let (mut objects, materials, _metadata) = GltfLoader::load(&gltf_path)?;

                    // Add materials to scene (with offset)
                    scene.materials.extend(materials);

                    // Update material indices and apply transform
                    for obj in &mut objects {
                        if let SceneObject::Mesh {
                            material,
                            transform: obj_transform,
                            ..
                        } = obj
                        {
                            // Offset material index
                            if let Some(mat_idx) = material {
                                *material = Some(*mat_idx + gltf_materials_offset);
                            }

                            // Apply GLTF model transform
                            *obj_transform = transform;
                        }
                    }

                    gltf_materials_offset = scene.materials.len();
                    expanded_objects.extend(objects);
                }
                other => {
                    expanded_objects.push(other);
                }
            }
        }

        scene.objects = expanded_objects;
        Ok(scene)
    }

    /// List available scenes in a directory
    pub fn list_scenes<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
        let dir = dir.as_ref();
        let mut scenes = Vec::new();

        if !dir.exists() {
            return Ok(scenes);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    scenes.push(name.to_string());
                }
            }
        }

        scenes.sort();
        Ok(scenes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_string() {
        let toml = r#"
            [metadata]
            name = "Test Scene"
            description = "A test scene"
            
            [camera]
            type = "perspective"
            position = [0.0, 0.0, 3.0]
            target = [0.0, 0.0, 0.0]
            fov = 45.0
        "#;

        let loader = SceneLoader::with_root("/tmp");
        let scene = loader.load_from_string(toml, None).unwrap();
        assert_eq!(scene.metadata.name, "Test Scene");
        assert_eq!(scene.metadata.description, "A test scene");
    }

    #[test]
    fn test_load_with_mesh() {
        let toml = r#"
            [metadata]
            name = "Triangle Scene"
            
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
        "#;

        let loader = SceneLoader::with_root("/tmp");
        let scene = loader.load_from_string(toml, None).unwrap();
        assert_eq!(scene.objects.len(), 1);

        match &scene.objects[0] {
            SceneObject::Mesh { name, geometry, .. } => {
                assert_eq!(name, "triangle");
                match geometry {
                    GeometryData::Inline { vertices, .. } => {
                        assert_eq!(vertices.len(), 3);
                    }
                    _ => panic!("Expected inline geometry"),
                }
            }
            _ => panic!("Expected mesh object"),
        }
    }

    #[test]
    fn test_invalid_scene() {
        let toml = r#"
            [metadata]
            name = ""
            
            [camera]
            type = "perspective"
            position = [0.0, 0.0, 3.0]
            target = [0.0, 0.0, 0.0]
        "#;

        let loader = SceneLoader::with_root("/tmp");
        assert!(loader.load_from_string(toml, None).is_err());
    }
}

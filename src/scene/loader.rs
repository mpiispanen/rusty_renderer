//! Scene loading from TOML files

use super::*;
use anyhow::{Context, Result};
use std::path::Path;

/// Scene loader for loading scenes from files
pub struct SceneLoader;

impl SceneLoader {
    /// Load a scene from a TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Scene> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read scene file: {}", path.display()))?;

        Self::load_from_string(&content)
    }

    /// Load a scene from a TOML string
    pub fn load_from_string(content: &str) -> Result<Scene> {
        let scene: Scene = toml::from_str(content).context("Failed to parse scene TOML")?;

        // Validate the scene
        scene.validate()?;

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

        let scene = SceneLoader::load_from_string(toml).unwrap();
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

        let scene = SceneLoader::load_from_string(toml).unwrap();
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

        assert!(SceneLoader::load_from_string(toml).is_err());
    }
}

//! Scene system for defining renderable scenes
//!
//! The scene system allows defining scenes through configuration files (TOML)
//! rather than hardcoding geometry in examples. This enables:
//! - Reusable scene definitions
//! - Easy testing with different scenes
//! - Separation of scene data from rendering logic
//! - Hot reloading (future)

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod loader;

pub use loader::SceneLoader;

/// Scene definition containing all scene data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scene {
    /// Scene metadata
    #[serde(default)]
    pub metadata: SceneMetadata,
    
    /// Objects in the scene
    #[serde(default)]
    pub objects: Vec<SceneObject>,
    
    /// Camera configuration
    pub camera: Camera,
    
    /// Optional lighting configuration
    #[serde(default)]
    pub lighting: Option<Lighting>,
}

/// Scene metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneMetadata {
    /// Scene name
    pub name: String,
    
    /// Scene description
    #[serde(default)]
    pub description: String,
    
    /// Author
    #[serde(default)]
    pub author: String,
}

/// Scene object (mesh, light, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SceneObject {
    /// Mesh object with inline geometry
    #[serde(rename = "mesh")]
    Mesh {
        name: String,
        geometry: GeometryData,
        #[serde(default)]
        transform: Transform,
    },
    
    /// glTF model reference
    #[serde(rename = "gltf")]
    GltfModel {
        name: String,
        path: String,
        #[serde(default)]
        transform: Transform,
    },
}

/// Geometry data for inline meshes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source")]
pub enum GeometryData {
    /// Inline vertex data
    #[serde(rename = "inline")]
    Inline {
        vertices: Vec<VertexData>,
        #[serde(default)]
        indices: Option<Vec<u32>>,
    },
    
    /// Reference to external file
    #[serde(rename = "file")]
    File { path: String },
}

/// Vertex data for inline geometry
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VertexData {
    pub position: [f32; 3],
    #[serde(default)]
    pub color: [f32; 3],
    #[serde(default)]
    pub normal: Option<[f32; 3]>,
    #[serde(default)]
    pub uv: Option<[f32; 2]>,
}

/// Transform (position, rotation, scale)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default)]
    pub rotation: [f32; 3], // Euler angles in degrees
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
}

fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        }
    }
}

/// Camera configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Camera {
    /// Perspective camera
    #[serde(rename = "perspective")]
    Perspective {
        position: [f32; 3],
        target: [f32; 3],
        #[serde(default = "default_up")]
        up: [f32; 3],
        #[serde(default = "default_fov")]
        fov: f32,
        #[serde(default = "default_near")]
        near: f32,
        #[serde(default = "default_far")]
        far: f32,
    },
    
    /// Free-fly camera (user controlled)
    #[serde(rename = "free_fly")]
    FreeFly {
        position: [f32; 3],
        #[serde(default = "default_yaw")]
        yaw: f32,
        #[serde(default = "default_pitch")]
        pitch: f32,
        #[serde(default = "default_fov")]
        fov: f32,
    },
}

fn default_up() -> [f32; 3] {
    [0.0, 1.0, 0.0]
}

fn default_fov() -> f32 {
    45.0
}

fn default_near() -> f32 {
    0.1
}

fn default_far() -> f32 {
    1000.0
}

fn default_yaw() -> f32 {
    -90.0
}

fn default_pitch() -> f32 {
    0.0
}

/// Lighting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lighting {
    #[serde(default)]
    pub ambient: [f32; 3],
    #[serde(default)]
    pub lights: Vec<Light>,
}

impl Default for Lighting {
    fn default() -> Self {
        Self {
            ambient: [0.1, 0.1, 0.1],
            lights: Vec::new(),
        }
    }
}

/// Light source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Light {
    /// Directional light (sun-like)
    #[serde(rename = "directional")]
    Directional {
        direction: [f32; 3],
        #[serde(default = "default_white")]
        color: [f32; 3],
        #[serde(default = "default_intensity")]
        intensity: f32,
    },
    
    /// Point light
    #[serde(rename = "point")]
    Point {
        position: [f32; 3],
        #[serde(default = "default_white")]
        color: [f32; 3],
        #[serde(default = "default_intensity")]
        intensity: f32,
    },
}

fn default_white() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_intensity() -> f32 {
    1.0
}

impl Scene {
    /// Validate the scene
    pub fn validate(&self) -> Result<()> {
        // Basic validation
        if self.metadata.name.is_empty() {
            anyhow::bail!("Scene must have a name");
        }
        
        // Validate objects
        for (i, obj) in self.objects.iter().enumerate() {
            match obj {
                SceneObject::Mesh { geometry, .. } => {
                    if let GeometryData::Inline { vertices, .. } = geometry {
                        if vertices.is_empty() {
                            anyhow::bail!("Object {} has no vertices", i);
                        }
                    }
                }
                SceneObject::GltfModel { path, .. } => {
                    if path.is_empty() {
                        anyhow::bail!("Object {} has empty path", i);
                    }
                }
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_transform() {
        let transform = Transform::default();
        assert_eq!(transform.position, [0.0, 0.0, 0.0]);
        assert_eq!(transform.scale, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_scene_validation() {
        let scene = Scene {
            metadata: SceneMetadata {
                name: "Test Scene".to_string(),
                ..Default::default()
            },
            objects: vec![],
            camera: Camera::Perspective {
                position: [0.0, 0.0, 3.0],
                target: [0.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                fov: 45.0,
                near: 0.1,
                far: 1000.0,
            },
            lighting: None,
        };
        
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn test_scene_validation_empty_name() {
        let scene = Scene {
            metadata: SceneMetadata {
                name: "".to_string(),
                ..Default::default()
            },
            objects: vec![],
            camera: Camera::Perspective {
                position: [0.0, 0.0, 3.0],
                target: [0.0, 0.0, 0.0],
                up: [0.0, 1.0, 0.0],
                fov: 45.0,
                near: 0.1,
                far: 1000.0,
            },
            lighting: None,
        };
        
        assert!(scene.validate().is_err());
    }
}

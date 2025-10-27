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

    /// Materials in the scene
    #[serde(default)]
    pub materials: Vec<Material>,
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
        /// Optional material reference (index into scene.materials)
        #[serde(default)]
        material: Option<usize>,
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
    #[serde(default = "default_vertex_color")]
    pub color: [f32; 3],
    #[serde(default)]
    pub normal: Option<[f32; 3]>,
    #[serde(default)]
    pub uv: Option<[f32; 2]>,
}

fn default_vertex_color() -> [f32; 3] {
    [1.0, 1.0, 1.0] // White - neutral for multiplying with texture/material colors
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

impl Transform {
    /// Calculate the model matrix from this transform
    pub fn matrix(&self) -> [[f32; 4]; 4] {
        use std::f32::consts::PI;

        // Convert rotation from degrees to radians
        let rx = self.rotation[0] * PI / 180.0;
        let ry = self.rotation[1] * PI / 180.0;
        let rz = self.rotation[2] * PI / 180.0;

        // Calculate sin/cos for each axis
        let (sx, cx) = rx.sin_cos();
        let (sy, cy) = ry.sin_cos();
        let (sz, cz) = rz.sin_cos();

        // Rotation matrices (ZYX order - typical for Euler angles)
        // R = Rz * Ry * Rx
        let r00 = cy * cz;
        let r01 = -cy * sz;
        let r02 = sy;
        
        let r10 = cx * sz + cz * sx * sy;
        let r11 = cx * cz - sx * sy * sz;
        let r12 = -cy * sx;
        
        let r20 = sx * sz - cx * cz * sy;
        let r21 = cz * sx + cx * sy * sz;
        let r22 = cx * cy;

        // Scale and combine with translation
        [
            [r00 * self.scale[0], r01 * self.scale[0], r02 * self.scale[0], 0.0],
            [r10 * self.scale[1], r11 * self.scale[1], r12 * self.scale[1], 0.0],
            [r20 * self.scale[2], r21 * self.scale[2], r22 * self.scale[2], 0.0],
            [self.position[0],    self.position[1],    self.position[2],    1.0],
        ]
    }

    /// Calculate the normal matrix (inverse transpose of upper 3x3)
    pub fn normal_matrix(&self) -> [[f32; 4]; 4] {
        let m = self.matrix();
        
        // Extract 3x3 rotation-scale part
        let m00 = m[0][0]; let m01 = m[0][1]; let m02 = m[0][2];
        let m10 = m[1][0]; let m11 = m[1][1]; let m12 = m[1][2];
        let m20 = m[2][0]; let m21 = m[2][1]; let m22 = m[2][2];
        
        // Calculate determinant
        let det = m00 * (m11 * m22 - m12 * m21)
                - m01 * (m10 * m22 - m12 * m20)
                + m02 * (m10 * m21 - m11 * m20);
        
        if det.abs() < 1e-6 {
            // Singular matrix, return identity
            return [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
        }
        
        let inv_det = 1.0 / det;
        
        // Calculate inverse transpose (transpose of inverse = inverse of transpose)
        // For normal transformation, we need inverse transpose
        let n00 = (m11 * m22 - m12 * m21) * inv_det;
        let n01 = (m02 * m21 - m01 * m22) * inv_det;
        let n02 = (m01 * m12 - m02 * m11) * inv_det;
        
        let n10 = (m12 * m20 - m10 * m22) * inv_det;
        let n11 = (m00 * m22 - m02 * m20) * inv_det;
        let n12 = (m02 * m10 - m00 * m12) * inv_det;
        
        let n20 = (m10 * m21 - m11 * m20) * inv_det;
        let n21 = (m01 * m20 - m00 * m21) * inv_det;
        let n22 = (m00 * m11 - m01 * m10) * inv_det;
        
        // Return as 4x4 matrix (padding with 0s and 1 for homogeneous coords)
        [
            [n00, n01, n02, 0.0],
            [n10, n11, n12, 0.0],
            [n20, n21, n22, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
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

/// Material definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    /// Material name
    pub name: String,
    
    /// Base color (RGB)
    #[serde(default = "default_white")]
    pub base_color: [f32; 3],
    
    /// Optional diffuse texture path (relative to scene file)
    #[serde(default)]
    pub diffuse_texture: Option<String>,
    
    /// Metallic factor (0.0 = dielectric, 1.0 = metal)
    #[serde(default)]
    pub metallic: f32,
    
    /// Roughness factor (0.0 = smooth, 1.0 = rough)
    #[serde(default = "default_roughness")]
    pub roughness: f32,
}

fn default_roughness() -> f32 {
    0.5
}

impl Default for Material {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            base_color: [0.8, 0.8, 0.8],
            diffuse_texture: None,
            metallic: 0.0,
            roughness: 0.5,
        }
    }
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
                SceneObject::Mesh { geometry, material, .. } => {
                    if let GeometryData::Inline { vertices, .. } = geometry {
                        if vertices.is_empty() {
                            anyhow::bail!("Object {i} has no vertices");
                        }
                    }
                    // Validate material reference
                    if let Some(mat_idx) = material {
                        if *mat_idx >= self.materials.len() {
                            anyhow::bail!("Object {i} references invalid material index {mat_idx}");
                        }
                    }
                }
                SceneObject::GltfModel { path, .. } => {
                    if path.is_empty() {
                        anyhow::bail!("Object {i} has empty path");
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
            materials: vec![],
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
            materials: vec![],
        };

        assert!(scene.validate().is_err());
    }
}

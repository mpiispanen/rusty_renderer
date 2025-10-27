//! Material system for managing rendering materials
//!
//! This module provides basic material support including:
//! - Material properties (color, metallic, roughness)
//! - Texture references
//! - GPU-side material data structures

use crate::scene::Material as SceneMaterial;
use std::io::Write;

/// GPU-friendly material data (std140 layout)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuMaterial {
    /// Base color (RGB) + padding
    pub base_color: [f32; 4],
    /// Metallic, roughness, has_diffuse_texture, padding
    pub properties: [f32; 4],
}

impl GpuMaterial {
    /// Create GPU material from scene material
    pub fn from_scene(material: &SceneMaterial) -> Self {
        let has_texture = if material.diffuse_texture.is_some() { 1.0 } else { 0.0 };
        
        let result = Self {
            base_color: [
                material.base_color[0],
                material.base_color[1],
                material.base_color[2],
                1.0, // alpha
            ],
            properties: [
                material.metallic,
                material.roughness,
                has_texture,
                0.0, // padding
            ],
        };
        
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("rusty_renderer_debug.log") {
            let _ = writeln!(f, "GpuMaterial created: base_color=[{:.2}, {:.2}, {:.2}], has_texture={}, texture_path={:?}",
                result.base_color[0], result.base_color[1], result.base_color[2], 
                has_texture, material.diffuse_texture);
        }
        log::info!("GpuMaterial created: base_color=[{:.2}, {:.2}, {:.2}], has_texture={}, texture_path={:?}",
            result.base_color[0], result.base_color[1], result.base_color[2], 
            has_texture, material.diffuse_texture);
        
        result
    }

    /// Convert to raw bytes for GPU upload
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }

    /// Size in bytes (32 bytes = 2 vec4s)
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_material_size() {
        // Should be 32 bytes (2 vec4s)
        assert_eq!(GpuMaterial::size(), 32);
    }

    #[test]
    fn test_gpu_material_from_scene() {
        let scene_mat = SceneMaterial {
            name: "test".to_string(),
            base_color: [1.0, 0.5, 0.25],
            diffuse_texture: Some("texture.png".to_string()),
            metallic: 0.8,
            roughness: 0.3,
        };

        let gpu_mat = GpuMaterial::from_scene(&scene_mat);
        
        assert_eq!(gpu_mat.base_color[0], 1.0);
        assert_eq!(gpu_mat.base_color[1], 0.5);
        assert_eq!(gpu_mat.base_color[2], 0.25);
        assert_eq!(gpu_mat.base_color[3], 1.0); // alpha
        
        assert_eq!(gpu_mat.properties[0], 0.8); // metallic
        assert_eq!(gpu_mat.properties[1], 0.3); // roughness
        assert_eq!(gpu_mat.properties[2], 1.0); // has texture
    }

    #[test]
    fn test_gpu_material_no_texture() {
        let scene_mat = SceneMaterial {
            name: "test".to_string(),
            base_color: [0.7, 0.7, 0.7],
            diffuse_texture: None,
            metallic: 0.0,
            roughness: 0.5,
        };

        let gpu_mat = GpuMaterial::from_scene(&scene_mat);
        assert_eq!(gpu_mat.properties[2], 0.0); // no texture
    }
}

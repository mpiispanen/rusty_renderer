//! Lighting system for forward rendering
//!
//! Handles light data structures and uniform buffers for GPU upload.
//! Supports directional lights, point lights, and ambient lighting.

use crate::scene::{Light, Lighting};
use glam::Vec3;

/// Maximum number of lights supported in shaders
pub const MAX_LIGHTS: usize = 8;

/// Light data for GPU (std140 layout)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuLight {
    /// Light type (0=directional, 1=point)
    pub light_type: u32,
    /// Padding for alignment
    pub _padding1: [u32; 3],
    /// Position (point) or direction (directional)
    pub position_or_direction: [f32; 4], // w unused for direction
    /// Color (RGB) + intensity (A)
    pub color_intensity: [f32; 4],
}

impl GpuLight {
    /// Create a directional light
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32) -> Self {
        let dir = direction.normalize();
        Self {
            light_type: 0,
            _padding1: [0; 3],
            position_or_direction: [dir.x, dir.y, dir.z, 0.0],
            color_intensity: [color.x, color.y, color.z, intensity],
        }
    }

    /// Create a point light
    pub fn point(position: Vec3, color: Vec3, intensity: f32) -> Self {
        Self {
            light_type: 1,
            _padding1: [0; 3],
            position_or_direction: [position.x, position.y, position.z, 1.0],
            color_intensity: [color.x, color.y, color.z, intensity],
        }
    }
}

// Safety: GpuLight is repr(C) with only primitive types
unsafe impl bytemuck::Pod for GpuLight {}
unsafe impl bytemuck::Zeroable for GpuLight {}

/// Lighting uniforms for GPU (std140 layout)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LightingUniforms {
    /// Ambient light color (RGB) + number of lights (A)
    pub ambient_light_count: [f32; 4],
    /// Array of lights
    pub lights: [GpuLight; MAX_LIGHTS],
}

impl LightingUniforms {
    /// Create from scene lighting configuration
    pub fn from_scene(lighting: &Lighting) -> Self {
        let mut uniforms = Self {
            ambient_light_count: [
                lighting.ambient[0],
                lighting.ambient[1],
                lighting.ambient[2],
                lighting.lights.len() as f32,
            ],
            lights: [GpuLight {
                light_type: 0,
                _padding1: [0; 3],
                position_or_direction: [0.0; 4],
                color_intensity: [0.0; 4],
            }; MAX_LIGHTS],
        };

        // Convert scene lights to GPU format
        for (i, light) in lighting.lights.iter().enumerate().take(MAX_LIGHTS) {
            uniforms.lights[i] = match light {
                Light::Directional {
                    direction,
                    color,
                    intensity,
                } => GpuLight::directional(
                    Vec3::from_array(*direction),
                    Vec3::from_array(*color),
                    *intensity,
                ),
                Light::Point {
                    position,
                    color,
                    intensity,
                } => GpuLight::point(
                    Vec3::from_array(*position),
                    Vec3::from_array(*color),
                    *intensity,
                ),
            };
        }

        if lighting.lights.len() > MAX_LIGHTS {
            log::warn!(
                "Scene has {} lights but only {} are supported. Excess lights ignored.",
                lighting.lights.len(),
                MAX_LIGHTS
            );
        }

        uniforms
    }

    /// Get as byte slice for buffer upload
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    /// Size in bytes
    pub fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

// Safety: LightingUniforms is repr(C) with only Pod types
unsafe impl bytemuck::Pod for LightingUniforms {}
unsafe impl bytemuck::Zeroable for LightingUniforms {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_light_directional() {
        let dir = Vec3::new(0.0, -1.0, 0.0);
        let color = Vec3::new(1.0, 1.0, 1.0);
        let light = GpuLight::directional(dir, color, 1.0);

        assert_eq!(light.light_type, 0);
        assert_eq!(light.color_intensity[3], 1.0);
    }

    #[test]
    fn test_gpu_light_point() {
        let pos = Vec3::new(5.0, 10.0, 0.0);
        let color = Vec3::new(1.0, 0.5, 0.0);
        let light = GpuLight::point(pos, color, 2.0);

        assert_eq!(light.light_type, 1);
        assert_eq!(light.position_or_direction[3], 1.0);
        assert_eq!(light.color_intensity[3], 2.0);
    }

    #[test]
    fn test_lighting_uniforms_from_scene() {
        let lighting = Lighting {
            ambient: [0.2, 0.2, 0.2],
            lights: vec![
                Light::Directional {
                    direction: [0.0, -1.0, 0.0],
                    color: [1.0, 1.0, 1.0],
                    intensity: 1.0,
                },
                Light::Point {
                    position: [5.0, 10.0, 0.0],
                    color: [1.0, 0.5, 0.0],
                    intensity: 2.0,
                },
            ],
        };

        let uniforms = LightingUniforms::from_scene(&lighting);

        assert_eq!(uniforms.ambient_light_count[0], 0.2);
        assert_eq!(uniforms.ambient_light_count[3], 2.0); // light count
        assert_eq!(uniforms.lights[0].light_type, 0); // directional
        assert_eq!(uniforms.lights[1].light_type, 1); // point
    }

    #[test]
    fn test_lighting_uniforms_size() {
        // Ensure uniform buffer size is what we expect for GPU
        let size = LightingUniforms::size();

        // 16 bytes (ambient + count) + 8 * 48 bytes (lights)
        // Each GpuLight: 4 bytes (type) + 12 bytes (padding) + 16 bytes (pos/dir) + 16 bytes (color/intensity) = 48 bytes
        assert_eq!(size, 16 + 8 * 48);
    }

    #[test]
    fn test_lighting_uniforms_max_lights() {
        let mut lights = Vec::new();
        for i in 0..10 {
            lights.push(Light::Point {
                position: [i as f32, 0.0, 0.0],
                color: [1.0, 1.0, 1.0],
                intensity: 1.0,
            });
        }

        let lighting = Lighting {
            ambient: [0.1, 0.1, 0.1],
            lights,
        };

        let uniforms = LightingUniforms::from_scene(&lighting);

        // Should only have MAX_LIGHTS (8)
        assert_eq!(uniforms.ambient_light_count[3], 10.0); // Original count stored
    }
}

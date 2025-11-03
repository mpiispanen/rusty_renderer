//! Lighting system

use crate::scene;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Light {
    pub light_type: u32,
    pub _padding1: u32,
    pub _padding2: u32,
    pub _padding3: u32,
    pub position_or_direction: [f32; 4],
    pub color_intensity: [f32; 4],
}

unsafe impl bytemuck::Pod for Light {}
unsafe impl bytemuck::Zeroable for Light {}

const MAX_LIGHTS: usize = 8;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LightingUniforms {
    pub ambient_light_count: [f32; 4], // RGB ambient + light count
    pub lights: [Light; MAX_LIGHTS],
}

unsafe impl bytemuck::Pod for LightingUniforms {}
unsafe impl bytemuck::Zeroable for LightingUniforms {}

pub struct Lighting {
    pub uniforms: LightingUniforms,
}

impl Lighting {
    pub fn new(scene_lighting: &scene::Lighting) -> Self {
        let mut lights_array = [Light {
            light_type: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
            position_or_direction: [0.0; 4],
            color_intensity: [0.0; 4],
        }; MAX_LIGHTS];

        let light_count = scene_lighting.lights.len().min(MAX_LIGHTS);
        for (i, scene_light) in scene_lighting.lights.iter().take(MAX_LIGHTS).enumerate() {
            lights_array[i] = match scene_light {
                scene::Light::Directional {
                    direction,
                    color,
                    intensity,
                } => Light {
                    light_type: 0, // LIGHT_DIRECTIONAL
                    _padding1: 0,
                    _padding2: 0,
                    _padding3: 0,
                    position_or_direction: [direction[0], direction[1], direction[2], 0.0],
                    color_intensity: [color[0], color[1], color[2], *intensity],
                },
                scene::Light::Point {
                    position,
                    color,
                    intensity,
                } => Light {
                    light_type: 1, // LIGHT_POINT
                    _padding1: 0,
                    _padding2: 0,
                    _padding3: 0,
                    position_or_direction: [position[0], position[1], position[2], 1.0],
                    color_intensity: [color[0], color[1], color[2], *intensity],
                },
            };
        }

        let uniforms = LightingUniforms {
            ambient_light_count: [
                scene_lighting.ambient[0],
                scene_lighting.ambient[1],
                scene_lighting.ambient[2],
                light_count as f32,
            ],
            lights: lights_array,
        };

        Self { uniforms }
    }

    pub fn buffer_data(&self) -> &[u8] {
        bytemuck::bytes_of(&self.uniforms)
    }
}

//! Vertex format definitions
//!
//! This module defines the standard vertex format used throughout the renderer.

use std::mem;

/// Standard vertex format with position, normal, UV, and color
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    /// Position in 3D space (x, y, z)
    pub position: [f32; 3],
    /// Normal vector (x, y, z)
    pub normal: [f32; 3],
    /// Texture coordinates (u, v)
    pub uv: [f32; 2],
    /// Vertex color (r, g, b, a)
    pub color: [f32; 4],
}

impl Vertex {
    /// Create a new vertex
    pub fn new(position: [f32; 3], normal: [f32; 3], uv: [f32; 2], color: [f32; 4]) -> Self {
        Self {
            position,
            normal,
            uv,
            color,
        }
    }

    /// Create a simple 2D vertex (for 2D rendering)
    pub fn new_2d(position: [f32; 2], color: [f32; 3]) -> Self {
        Self {
            position: [position[0], position[1], 0.0],
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
            color: [color[0], color[1], color[2], 1.0],
        }
    }

    /// Get the size of a vertex in bytes
    pub const fn size() -> usize {
        mem::size_of::<Self>()
    }

    /// Get the stride between vertices in bytes
    pub const fn stride() -> u32 {
        Self::size() as u32
    }

    /// Get vertex attribute descriptions for Vulkan
    pub fn attribute_descriptions_vulkan() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute {
                location: 0,
                binding: 0,
                format: VertexFormat::Float3,
                offset: 0, // position offset
            },
            VertexAttribute {
                location: 1,
                binding: 0,
                format: VertexFormat::Float3,
                offset: 12, // normal offset (after position)
            },
            VertexAttribute {
                location: 2,
                binding: 0,
                format: VertexFormat::Float2,
                offset: 24, // uv offset (after position + normal)
            },
            VertexAttribute {
                location: 3,
                binding: 0,
                format: VertexFormat::Float4,
                offset: 32, // color offset (after position + normal + uv)
            },
        ]
    }
}

/// Vertex attribute format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexFormat {
    Float,
    Float2,
    Float3,
    Float4,
    UInt,
    UInt2,
    UInt3,
    UInt4,
}

impl VertexFormat {
    /// Get the size of this format in bytes
    pub fn size(&self) -> u32 {
        match self {
            VertexFormat::Float => 4,
            VertexFormat::Float2 => 8,
            VertexFormat::Float3 => 12,
            VertexFormat::Float4 => 16,
            VertexFormat::UInt => 4,
            VertexFormat::UInt2 => 8,
            VertexFormat::UInt3 => 12,
            VertexFormat::UInt4 => 16,
        }
    }
}

/// Vertex attribute description
#[derive(Debug, Clone)]
pub struct VertexAttribute {
    /// Shader location/binding index
    pub location: u32,
    /// Vertex buffer binding index
    pub binding: u32,
    /// Format of the attribute
    pub format: VertexFormat,
    /// Offset in bytes from the start of the vertex
    pub offset: u32,
}

/// Vertex buffer binding description
#[derive(Debug, Clone)]
pub struct VertexBufferLayout {
    /// Stride between vertices in bytes
    pub stride: u32,
    /// Input rate (per-vertex or per-instance)
    pub input_rate: VertexInputRate,
}

/// Vertex input rate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexInputRate {
    /// One entry per vertex
    Vertex,
    /// One entry per instance
    Instance,
}

impl Default for VertexBufferLayout {
    fn default() -> Self {
        Self {
            stride: Vertex::stride(),
            input_rate: VertexInputRate::Vertex,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_size() {
        // Vertex should be: 3*4 + 3*4 + 2*4 + 4*4 = 12 + 12 + 8 + 16 = 48 bytes
        assert_eq!(Vertex::size(), 48);
        assert_eq!(Vertex::stride(), 48);
    }

    #[test]
    fn test_vertex_creation() {
        let v = Vertex::new(
            [1.0, 2.0, 3.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5],
            [1.0, 0.0, 0.0, 1.0],
        );
        assert_eq!(v.position, [1.0, 2.0, 3.0]);
        assert_eq!(v.normal, [0.0, 1.0, 0.0]);
        assert_eq!(v.uv, [0.5, 0.5]);
        assert_eq!(v.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_vertex_2d() {
        let v = Vertex::new_2d([1.0, 2.0], [1.0, 0.0, 0.0]);
        assert_eq!(v.position, [1.0, 2.0, 0.0]);
        assert_eq!(v.normal, [0.0, 0.0, 1.0]);
        assert_eq!(v.color, [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_attribute_offsets() {
        let attrs = Vertex::attribute_descriptions_vulkan();
        assert_eq!(attrs.len(), 4);
        assert_eq!(attrs[0].offset, 0); // position
        assert_eq!(attrs[1].offset, 12); // normal
        assert_eq!(attrs[2].offset, 24); // uv
        assert_eq!(attrs[3].offset, 32); // color
    }
}

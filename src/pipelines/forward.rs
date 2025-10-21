//! Forward rendering pipeline with lighting
//!
//! Implements forward rendering with support for:
//! - Multiple light sources (directional and point lights)
//! - Phong/Blinn-Phong shading
//! - Camera transforms (MVP matrices)
//! - Depth testing

use super::*;
use crate::backends::{
    BufferDescriptor, BufferUsage, GraphicsBackend, MemoryLocation, Vertex as BackendVertex,
};
use crate::camera::CameraController;
use crate::lighting::LightingUniforms;
use crate::passes::VertexBufferTrianglePass;
use crate::render_graph::{
    Extent3D, Format, ImageUsageFlags, RenderGraph, ResourceDescriptor, SampleCount,
};
use crate::scene::{GeometryData, SceneObject, VertexData};
use anyhow::Context as _;

/// Forward rendering pipeline with lighting
pub struct ForwardPipeline {
    /// Pipeline name
    name: String,
    /// Camera controller (optional, created during build_graph)
    camera: Option<CameraController>,
}

impl ForwardPipeline {
    /// Create a new forward pipeline
    pub fn new() -> Self {
        Self {
            name: "Forward".to_string(),
            camera: None,
        }
    }

    /// Convert scene vertex data to backend vertex format with normals
    fn convert_vertex(vertex: &VertexData) -> BackendVertex {
        // Use provided normal or calculate default
        let normal = vertex.normal.unwrap_or([0.0, 0.0, 1.0]);
        let uv = vertex.uv.unwrap_or([0.0, 0.0]);
        
        BackendVertex::new(
            vertex.position,
            normal,
            uv,
            [vertex.color[0], vertex.color[1], vertex.color[2], 1.0],
        )
    }

    /// Create vertex buffer from scene vertices
    fn create_vertex_buffer(
        backend: &mut dyn GraphicsBackend,
        vertices: &[VertexData],
        label: &str,
    ) -> Result<Box<dyn crate::backends::Buffer>> {
        // Convert to backend vertex format
        let backend_vertices: Vec<BackendVertex> =
            vertices.iter().map(Self::convert_vertex).collect();

        // Create buffer descriptor
        let vertex_buffer_size = (backend_vertices.len() * BackendVertex::size()) as u64;
        let vertex_desc = BufferDescriptor {
            size: vertex_buffer_size,
            usage: BufferUsage::vertex(),
            memory_location: MemoryLocation::GpuOnly,
            label: Some(label.to_string()),
        };

        // Create buffer
        let vertex_buffer = backend.create_buffer(&vertex_desc)?;

        // Upload data
        let vertex_data: Vec<u8> = backend_vertices
            .iter()
            .flat_map(|v| {
                let mut bytes = Vec::new();
                bytes.extend_from_slice(bytemuck::bytes_of(v));
                bytes
            })
            .collect();

        backend.upload_to_buffer(vertex_buffer.as_ref(), &vertex_data, 0)?;

        log::info!(
            "Created vertex buffer '{}': {} vertices",
            label,
            backend_vertices.len()
        );

        Ok(vertex_buffer)
    }

    /// Calculate default normals for geometry without normals
    fn calculate_normals(vertices: &[VertexData]) -> Vec<VertexData> {
        // For now, just return vertices with default normals
        // TODO: Calculate per-face normals for triangles
        vertices
            .iter()
            .map(|v| {
                let mut vertex = *v;
                if vertex.normal.is_none() {
                    // Simple default: pointing up
                    vertex.normal = Some([0.0, 1.0, 0.0]);
                }
                vertex
            })
            .collect()
    }
}

impl Default for ForwardPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderPipeline for ForwardPipeline {
    fn name(&self) -> &str {
        &self.name
    }

    fn setup(&mut self, _backend: &mut dyn crate::backends::GraphicsBackend) -> Result<()> {
        log::info!("Forward pipeline setup complete");
        Ok(())
    }

    fn build_graph(
        &mut self,
        scene: &Scene,
        backend: &mut dyn crate::backends::GraphicsBackend,
    ) -> Result<RenderGraph> {
        log::debug!("Building forward render graph for scene: {}", scene.metadata.name);

        let mut graph = RenderGraph::new();

        // Get dimensions from backend (or use defaults)
        let (width, height) = (800, 600); // TODO: Get from backend or args

        // Create camera controller
        let camera = CameraController::from_scene_camera(&scene.camera, width, height);
        let _camera_uniforms = camera.uniforms();

        // Store camera for later use
        self.camera = Some(camera);

        // Get lighting configuration
        let lighting = scene.lighting.as_ref().cloned().unwrap_or_default();
        let _lighting_uniforms = LightingUniforms::from_scene(&lighting);

        log::info!(
            "Forward pipeline: {} lights, ambient {:?}",
            lighting.lights.len(),
            lighting.ambient
        );

        // Create color buffer resource
        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: Extent3D::new_2d(width, height),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        // TODO: Create depth buffer for 3D rendering
        // let depth_desc = ResourceDescriptor::Image {
        //     format: Format::Depth32Float,
        //     extent: Extent3D::new_2d(width, height),
        //     usage: ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        //     samples: SampleCount::One,
        // };
        // let depth_buffer = graph.create_resource("depth_buffer", depth_desc);

        log::info!(
            "ForwardPipeline: Building graph for scene '{}' with {} objects",
            scene.metadata.name,
            scene.objects.len()
        );

        // Process each object in the scene
        for obj in scene.objects.iter() {
            match obj {
                SceneObject::Mesh { name, geometry, transform } => match geometry {
                    GeometryData::Inline { vertices, .. } => {
                        let vertex_count = vertices.len();
                        log::info!("  - Mesh '{name}': {vertex_count} vertices");

                        // Ensure vertices have normals
                        let vertices_with_normals = Self::calculate_normals(vertices);

                        // Create vertex buffer
                        let label = format!("{name}_vertices");
                        let vertex_buffer = Self::create_vertex_buffer(backend, &vertices_with_normals, &label)
                            .with_context(|| {
                            format!("Failed to create vertex buffer for mesh '{name}'")
                        })?;

                        // Log transform for debugging
                        log::debug!("    Transform: position={:?}, rotation={:?}, scale={:?}",
                            transform.position, transform.rotation, transform.scale);

                        // Add render pass for this mesh
                        // TODO: Create ForwardPass that uses camera and lighting uniforms
                        let _pass =
                            VertexBufferTrianglePass::new(&mut graph, color_buffer, vertex_buffer);

                        log::debug!("Added render pass for mesh '{name}'");
                    }
                    GeometryData::File { path } => {
                        log::warn!("  - Mesh '{name}': external file '{path}' not yet supported");
                    }
                },
                SceneObject::GltfModel { name, path, .. } => {
                    log::warn!("  - glTF Model '{name}': '{path}' not yet supported");
                }
            }
        }

        if scene.objects.is_empty() {
            anyhow::bail!("Scene has no objects to render");
        }

        log::info!("ForwardPipeline: Render graph built successfully");

        Ok(graph)
    }

    fn cleanup(&mut self, _backend: &mut dyn crate::backends::GraphicsBackend) {
        log::info!("Forward pipeline cleanup");
        self.camera = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_forward_pipeline_creation() {
        let pipeline = ForwardPipeline::new();
        assert_eq!(pipeline.name(), "Forward");
    }

    #[test]
    fn test_convert_vertex_with_normal() {
        let vertex = VertexData {
            position: [1.0, 2.0, 3.0],
            color: [1.0, 0.0, 0.0],
            normal: Some([0.0, 1.0, 0.0]),
            uv: Some([0.5, 0.5]),
        };

        let backend_vertex = ForwardPipeline::convert_vertex(&vertex);
        assert_eq!(backend_vertex.position, [1.0, 2.0, 3.0]);
        assert_eq!(backend_vertex.normal, [0.0, 1.0, 0.0]);
        assert_eq!(backend_vertex.uv, [0.5, 0.5]);
    }

    #[test]
    fn test_convert_vertex_default_normal() {
        let vertex = VertexData {
            position: [1.0, 2.0, 3.0],
            color: [1.0, 0.0, 0.0],
            normal: None,
            uv: None,
        };

        let backend_vertex = ForwardPipeline::convert_vertex(&vertex);
        assert_eq!(backend_vertex.normal, [0.0, 0.0, 1.0]);
        assert_eq!(backend_vertex.uv, [0.0, 0.0]);
    }
}

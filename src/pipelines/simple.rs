//! Simple rendering pipeline
//!
//! Renders scenes with vertex colors, no lighting. This is the most basic
//! pipeline for testing and simple visualization.

use super::*;
use crate::backends::{
    BufferDescriptor, BufferUsage, GraphicsBackend, MemoryLocation, Vertex as BackendVertex,
};
use crate::passes::VertexBufferTrianglePass;
use crate::render_graph::{
    Extent3D, ExtentMode, Format, ImageUsageFlags, RenderGraph, ResourceDescriptor, SampleCount,
    ShaderDescriptor, ShaderStage,
};
use crate::scene::{GeometryData, SceneObject, VertexData};
use anyhow::Context as _;

/// Simple pipeline for vertex-colored geometry
pub struct SimplePipeline {
    /// Pipeline name
    name: String,
}

impl SimplePipeline {
    /// Create a new simple pipeline
    pub fn new() -> Self {
        Self {
            name: "Simple".to_string(),
        }
    }

    /// Convert scene vertex data to backend vertex format
    fn convert_vertex(vertex: &VertexData) -> BackendVertex {
        BackendVertex::new_2d([vertex.position[0], vertex.position[1]], vertex.color)
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

    /// Create index buffer from scene indices
    fn create_index_buffer(
        backend: &mut dyn GraphicsBackend,
        indices: &[u32],
        label: &str,
    ) -> Result<Box<dyn crate::backends::Buffer>> {
        // Create buffer descriptor for index buffer
        let index_buffer_size = std::mem::size_of_val(indices) as u64;
        let index_desc = BufferDescriptor {
            size: index_buffer_size,
            usage: BufferUsage::index(),
            memory_location: MemoryLocation::GpuOnly,
            label: Some(label.to_string()),
        };

        // Create buffer
        let index_buffer = backend.create_buffer(&index_desc)?;

        // Upload index data
        let index_data: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();

        backend.upload_to_buffer(index_buffer.as_ref(), &index_data, 0)?;

        log::info!(
            "Created index buffer '{}': {} indices",
            label,
            indices.len()
        );

        Ok(index_buffer)
    }
}

impl Default for SimplePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderPipeline for SimplePipeline {
    fn name(&self) -> &str {
        &self.name
    }

    fn setup(&mut self, _backend: &mut dyn crate::backends::GraphicsBackend) -> Result<()> {
        log::info!("Simple pipeline setup complete");
        Ok(())
    }

    fn build_graph(
        &mut self,
        scene: &Scene,
        backend: &mut dyn crate::backends::GraphicsBackend,
    ) -> Result<RenderGraph> {
        log::debug!("Building render graph for scene: {}", scene.metadata.name);

        let mut graph = RenderGraph::new();

        // Register shaders needed for simple pipeline
        graph.register_shader(
            "simple_vertex.vert",
            ShaderDescriptor::from_file("shaders/hlsl/simple_vertex.hlsl", ShaderStage::Vertex)
                .with_entry_point("VSMain"),
        );
        graph.register_shader(
            "simple_vertex.frag",
            ShaderDescriptor::from_file("shaders/hlsl/simple_vertex.hlsl", ShaderStage::Fragment)
                .with_entry_point("PSMain"),
        );

        // Get dimensions from backend (or use defaults)
        let (width, height) = (800, 600); // TODO: Get from backend or args

        // Create color buffer resource
        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(width, height)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        log::info!(
            "SimplePipeline: Building graph for scene '{}' with {} objects",
            scene.metadata.name,
            scene.objects.len()
        );

        // Process each object in the scene
        for obj in scene.objects.iter() {
            match obj {
                SceneObject::Mesh { name, geometry, .. } => match geometry {
                    GeometryData::Inline { vertices, indices } => {
                        let vertex_count = vertices.len();
                        log::info!("  - Mesh '{name}': {vertex_count} vertices");

                        // Create vertex buffer
                        let label = format!("{name}_vertices");
                        let vertex_buffer = Self::create_vertex_buffer(backend, vertices, &label)
                            .with_context(|| {
                            format!("Failed to create vertex buffer for mesh '{name}'")
                        })?;

                        // Create index buffer if indices are provided
                        let (index_buffer, draw_count) = if let Some(idx) = indices {
                            let index_label = format!("{name}_indices");
                            let ib = Self::create_index_buffer(backend, idx, &index_label)
                                .with_context(|| {
                                    format!("Failed to create index buffer for mesh '{name}'")
                                })?;
                            let count = idx.len() as u32;
                            log::info!("  - Mesh '{name}': {count} indices");
                            (Some(ib), count)
                        } else {
                            (None, vertex_count as u32)
                        };

                        // Add render pass for this mesh
                        let _pass = VertexBufferTrianglePass::new(
                            &mut graph,
                            color_buffer,
                            vertex_buffer,
                            index_buffer,
                            draw_count,
                        );

                        log::debug!("Added VertexBufferTrianglePass for mesh '{name}'");
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

        log::info!("SimplePipeline: Render graph built successfully");

        Ok(graph)
    }

    fn cleanup(&mut self, _backend: &mut dyn crate::backends::GraphicsBackend) {
        log::info!("Simple pipeline cleanup");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_pipeline_creation() {
        let pipeline = SimplePipeline::new();
        assert_eq!(pipeline.name(), "Simple");
    }
}

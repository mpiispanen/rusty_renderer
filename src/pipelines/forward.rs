//! Forward rendering pipeline with lighting
//!
//! Implements forward rendering with support for:
//! - Multiple light sources (directional and point lights)
//! - Phong/Blinn-Phong shading
//! - Camera transforms (MVP matrices)
//! - Depth testing

use super::*;
use crate::backends::{
    BufferDescriptor, BufferUsage, GraphicsBackend, MemoryLocation, Texture, TextureDescriptor,
    TextureFormat, TextureUsage, Vertex as BackendVertex,
};
use crate::camera::{CameraController, CameraUniforms};
use crate::lighting::LightingUniforms;
use crate::materials::GpuMaterial;
use crate::render_graph::{
    Extent3D, ExtentMode, Format, ImageUsageFlags, RenderGraph, ResourceDescriptor, SampleCount,
};
use crate::resources::TextureLoader;
use crate::scene::{GeometryData, SceneObject, VertexData};
use anyhow::Context as _;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

/// Forward rendering pipeline with lighting
pub struct ForwardPipeline {
    /// Pipeline name
    name: String,
    /// Camera controller (optional, created during build_graph)
    camera: Option<CameraController>,
    /// Default white 1x1 texture for objects without textures
    default_texture: Option<Arc<Box<dyn Texture>>>,
    /// Default material for objects without materials
    default_material_buffer: Option<Arc<Box<dyn crate::backends::Buffer>>>,
}

impl ForwardPipeline {
    /// Create a new forward pipeline
    pub fn new() -> Self {
        Self {
            name: "Forward".to_string(),
            camera: None,
            default_texture: None,
            default_material_buffer: None,
        }
    }

    /// Convert scene vertex data to backend vertex format with normals
    fn convert_vertex(vertex: &VertexData) -> BackendVertex {
        // Use provided normal or calculate default
        let normal = vertex.normal;
        let uv = vertex.uv;

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
            memory_location: MemoryLocation::CpuToGpu,
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

        Ok(vertex_buffer)
    }

    /// Create uniform buffer for camera
    fn create_camera_buffer(
        backend: &mut dyn GraphicsBackend,
        camera_uniforms: &CameraUniforms,
        label: &str,
    ) -> Result<Box<dyn crate::backends::Buffer>> {
        let buffer_size = std::mem::size_of::<CameraUniforms>() as u64;
        let buffer_desc = BufferDescriptor {
            size: buffer_size,
            usage: BufferUsage::uniform(),
            memory_location: MemoryLocation::CpuToGpu,
            label: Some(label.to_string()),
        };

        let buffer = backend.create_buffer(&buffer_desc)?;

        // Upload camera data
        let data = bytemuck::bytes_of(camera_uniforms);
        backend.upload_to_buffer(buffer.as_ref(), data, 0)?;

        Ok(buffer)
    }

    /// Create uniform buffer for lighting
    fn create_lighting_buffer(
        backend: &mut dyn GraphicsBackend,
        lighting_uniforms: &LightingUniforms,
        label: &str,
    ) -> Result<Box<dyn crate::backends::Buffer>> {
        let buffer_size = std::mem::size_of::<LightingUniforms>() as u64;
        let buffer_desc = BufferDescriptor {
            size: buffer_size,
            usage: BufferUsage::uniform(),
            memory_location: MemoryLocation::CpuToGpu,
            label: Some(label.to_string()),
        };

        let buffer = backend.create_buffer(&buffer_desc)?;

        // Upload lighting data
        let data = bytemuck::bytes_of(lighting_uniforms);
        backend.upload_to_buffer(buffer.as_ref(), data, 0)?;

        Ok(buffer)
    }

    /// Create uniform buffer for material properties
    fn create_material_buffer(
        backend: &mut dyn GraphicsBackend,
        material: &GpuMaterial,
        label: &str,
    ) -> Result<Box<dyn crate::backends::Buffer>> {
        let buffer_size = GpuMaterial::size() as u64;
        let buffer_desc = BufferDescriptor {
            size: buffer_size,
            usage: BufferUsage::uniform(),
            memory_location: MemoryLocation::CpuToGpu,
            label: Some(label.to_string()),
        };

        let buffer = backend.create_buffer(&buffer_desc)?;

        // Upload material data
        let data = material.as_bytes();

        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            let _ = writeln!(f, "Uploading material to buffer: base_color=[{:.2}, {:.2}, {:.2}, {:.2}], properties=[{:.2}, {:.2}, {:.2}, {:.2}]",
                material.base_color[0], material.base_color[1], material.base_color[2], material.base_color[3],
                material.properties[0], material.properties[1], material.properties[2], material.properties[3]);
        }

        backend.upload_to_buffer(buffer.as_ref(), data, 0)?;

        Ok(buffer)
    }

    /// Load a texture from file
    fn load_texture(
        backend: &mut dyn GraphicsBackend,
        path: &str,
        label: &str,
    ) -> Result<Box<dyn Texture>> {
        // Load image from file
        let image = TextureLoader::load_from_file(Path::new(path))
            .with_context(|| format!("Failed to load texture from '{path}'"))?;

        // Create texture descriptor with initial data
        let texture_desc = TextureDescriptor {
            width: image.width,
            height: image.height,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::sampled(), // Use helper that sets transfer_dst automatically
            mip_levels: 1,
            initial_data: Some(&image.data),
            label: Some(label.to_string()),
        };

        // Create texture (will upload initial_data automatically)
        let texture = backend.create_texture(&texture_desc)?;

        Ok(texture)
    }

    /// Create a default 1x1 white texture
    fn create_default_texture(backend: &mut dyn GraphicsBackend) -> Result<Box<dyn Texture>> {
        // Create 1x1 white RGBA texture
        let white_pixel = vec![255u8, 255, 255, 255]; // RGBA white

        let texture_desc = TextureDescriptor {
            width: 1,
            height: 1,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::sampled(),
            mip_levels: 1,
            initial_data: Some(&white_pixel),
            label: Some("default_white_texture".to_string()),
        };

        let texture = backend.create_texture(&texture_desc)?;
        log::info!("Created default 1x1 white texture");

        Ok(texture)
    }

    /// Create a default material buffer
    fn create_default_material(
        backend: &mut dyn GraphicsBackend,
    ) -> Result<Box<dyn crate::backends::Buffer>> {
        let default_material = GpuMaterial {
            base_color: [1.0, 1.0, 1.0, 1.0], // White
            properties: [0.0, 0.5, 0.0, 0.0], // No metallic, medium roughness, no texture
        };

        let buffer_size = GpuMaterial::size() as u64;
        let buffer_desc = BufferDescriptor {
            size: buffer_size,
            usage: BufferUsage::uniform(),
            memory_location: MemoryLocation::CpuToGpu,
            label: Some("default_material".to_string()),
        };

        let buffer = backend.create_buffer(&buffer_desc)?;
        let data = default_material.as_bytes();
        backend.upload_to_buffer(buffer.as_ref(), data, 0)?;

        log::info!("Created default material buffer");

        Ok(buffer)
    }

    /// Calculate default normals for geometry without normals
    fn calculate_normals(vertices: &[VertexData]) -> Vec<VertexData> {
        // For now, just return vertices with default normals
        // TODO: Calculate per-face normals for triangles
        vertices.to_vec()
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
        log::debug!(
            "Building forward render graph for scene: {}",
            scene.metadata.name
        );

        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            let _ = writeln!(
                f,
                "ForwardPipeline::build_graph ENTERED for scene '{}'",
                scene.metadata.name
            );
            let _ = f.flush();
        }

        let mut graph = RenderGraph::new();

        // Note: Shaders are registered centrally in App::register_shaders()
        // before this method is called. We just reference them by name.
        log::info!("Using pre-registered forward shaders from shader registry");

        // Get dimensions from backend (or use defaults)
        let (width, height) = (800, 600); // TODO: Get from backend or args

        // Create camera controller
        let camera = CameraController::from_scene_camera(&scene.camera, width, height);
        let camera_uniforms = camera.uniforms();

        // Store camera for later use
        self.camera = Some(camera);

        // Create camera uniform buffer
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            let _ = writeln!(f, "About to create camera uniform buffer");
            let _ = writeln!(f, "Camera view_proj matrix:");
            for (i, row) in camera_uniforms.view_proj.iter().enumerate() {
                let _ = writeln!(
                    f,
                    "  row{}: [{:.4}, {:.4}, {:.4}, {:.4}]",
                    i, row[0], row[1], row[2], row[3]
                );
            }
            let _ = f.flush();
        }
        let camera_buffer =
            Self::create_camera_buffer(backend, &camera_uniforms, "camera_uniforms")
                .context("Failed to create camera uniform buffer")?;
        let camera_buffer = Arc::new(camera_buffer);

        if let Ok(mut f) = std::fs::OpenOptions::new()
            .append(true)
            .open("rusty_renderer_debug.log")
        {
            let _ = writeln!(f, "Camera uniform buffer created");
            let _ = f.flush();
        }

        log::info!("Created camera uniform buffer");

        // Get lighting configuration
        let lighting = scene.lighting.as_ref().cloned().unwrap_or_default();
        let lighting_uniforms = LightingUniforms::from_scene(&lighting);

        log::info!(
            "Lighting uniforms - ambient: [{:.2}, {:.2}, {:.2}], light_count: {}",
            lighting_uniforms.ambient_light_count[0],
            lighting_uniforms.ambient_light_count[1],
            lighting_uniforms.ambient_light_count[2],
            lighting_uniforms.ambient_light_count[3]
        );

        // Log first light for debugging
        if lighting_uniforms.ambient_light_count[3] > 0.0 {
            let light = &lighting_uniforms.lights[0];
            log::info!(
                "Light 0 - type: {}, dir/pos: [{:.2}, {:.2}, {:.2}], color: [{:.2}, {:.2}, {:.2}], intensity: {:.2}",
                light.light_type,
                light.position_or_direction[0],
                light.position_or_direction[1],
                light.position_or_direction[2],
                light.color_intensity[0],
                light.color_intensity[1],
                light.color_intensity[2],
                light.color_intensity[3]
            );
        }

        // Log second light too
        if lighting_uniforms.ambient_light_count[3] > 1.0 {
            let light = &lighting_uniforms.lights[1];
            log::info!(
                "Light 1 - type: {}, dir/pos: [{:.2}, {:.2}, {:.2}], color: [{:.2}, {:.2}, {:.2}], intensity: {:.2}",
                light.light_type,
                light.position_or_direction[0],
                light.position_or_direction[1],
                light.position_or_direction[2],
                light.color_intensity[0],
                light.color_intensity[1],
                light.color_intensity[2],
                light.color_intensity[3]
            );
        }

        // Create lighting uniform buffer
        let lighting_buffer =
            Self::create_lighting_buffer(backend, &lighting_uniforms, "lighting_uniforms")
                .context("Failed to create lighting uniform buffer")?;
        let lighting_buffer = Arc::new(lighting_buffer);

        log::info!(
            "Forward pipeline: {} lights, ambient {:?}",
            lighting.lights.len(),
            lighting.ambient
        );

        // Create color buffer resource
        let color_desc = ResourceDescriptor::Image {
            format: Format::Bgra8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(width, height)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color_buffer = graph.create_resource("color_buffer", color_desc);

        // TODO: Create depth buffer for 3D rendering
        // let depth_desc = ResourceDescriptor::Image {
        //     format: Format::Depth32Float,
        //     extent: ExtentMode::Absolute(Extent3D::new_2d(width, height)),
        //     usage: ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        //     samples: SampleCount::One,
        //     mip_levels: 1,
        // };
        // let depth_buffer = graph.create_resource("depth_buffer", depth_desc);

        log::info!(
            "ForwardPipeline: Building graph for scene '{}' with {} objects",
            scene.metadata.name,
            scene.objects.len()
        );

        // Create default texture and material if not already created
        if self.default_texture.is_none() {
            log::info!("Creating default fallback texture and material");
            let default_tex = Self::create_default_texture(backend)
                .context("Failed to create default texture")?;
            self.default_texture = Some(Arc::new(default_tex));

            let default_mat = Self::create_default_material(backend)
                .context("Failed to create default material")?;
            self.default_material_buffer = Some(Arc::new(default_mat));
        }

        // Process each object in the scene
        for obj in scene.objects.iter() {
            match obj {
                SceneObject::Mesh {
                    name,
                    geometry,
                    transform,
                    material,
                } => match geometry {
                    GeometryData::Inline { vertices, .. } => {
                        let vertex_count = vertices.len();
                        log::info!("  - Mesh '{name}': {vertex_count} vertices");

                        // Ensure vertices have normals
                        let vertices_with_normals = Self::calculate_normals(vertices);

                        // Create vertex buffer
                        let label = format!("{name}_vertices");
                        let vertex_buffer =
                            Self::create_vertex_buffer(backend, &vertices_with_normals, &label)
                                .with_context(|| {
                                    format!("Failed to create vertex buffer for mesh '{name}'")
                                })?;

                        // Load material and texture if specified, otherwise use defaults
                        let (material_buffer, texture) = if let Some(mat_idx) = material {
                            if *mat_idx < scene.materials.len() {
                                let scene_material = &scene.materials[*mat_idx];
                                log::info!(
                                    "  - Using material '{}' (index {})",
                                    scene_material.name,
                                    mat_idx
                                );

                                // Create material uniform buffer
                                let gpu_material = GpuMaterial::from_scene(scene_material);
                                let material_buffer = Self::create_material_buffer(
                                    backend,
                                    &gpu_material,
                                    &format!("{name}_material"),
                                )
                                .context("Failed to create material buffer")?;

                                // Load texture if specified, otherwise use default white texture
                                let texture = if let Some(ref texture_path) =
                                    scene_material.diffuse_texture
                                {
                                    log::info!("  - Loading texture: {texture_path}");
                                    match Self::load_texture(
                                        backend,
                                        texture_path,
                                        &format!("{name}_diffuse"),
                                    ) {
                                        Ok(tex) => Some(Arc::new(tex)),
                                        Err(e) => {
                                            log::warn!("  - Failed to load texture '{texture_path}': {e}, using default");
                                            self.default_texture.clone()
                                        }
                                    }
                                } else {
                                    log::info!("  - No texture specified, using default white");
                                    self.default_texture.clone()
                                };

                                (Some(Arc::new(material_buffer)), texture)
                            } else {
                                log::warn!("  - Invalid material index {mat_idx}, using defaults");
                                (
                                    self.default_material_buffer.clone(),
                                    self.default_texture.clone(),
                                )
                            }
                        } else {
                            log::info!("  - No material specified, using defaults");
                            (
                                self.default_material_buffer.clone(),
                                self.default_texture.clone(),
                            )
                        };

                        // Add forward rendering pass with camera and lighting
                        let vertex_count = vertices_with_normals.len() as u32;

                        // Debug: log which material buffer we're using
                        if let Ok(mut f) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("rusty_renderer_debug.log")
                        {
                            use std::io::Write;
                            match &material_buffer {
                                Some(buf) => {
                                    let ptr = buf.as_ref().as_ref()
                                        as *const dyn crate::backends::Buffer
                                        as *const std::ffi::c_void;
                                    let _ = writeln!(f, "Creating ForwardDeclarativePass for '{name}' with material buffer at {ptr:p}");
                                }
                                None => {
                                    let _ = writeln!(f, "Creating ForwardDeclarativePass for '{name}' with NO material buffer (using default)");
                                }
                            }
                        }

                        // Use the new ForwardRenderPass builder API
                        let mut builder = crate::passes::ForwardRenderPass::builder()
                            .color_output(color_buffer)
                            .vertex_buffer(vertex_buffer)
                            .camera_buffer(camera_buffer.clone())
                            .lighting_buffer(lighting_buffer.clone())
                            .transform(*transform)
                            .vertex_count(vertex_count)
                            .with_name(format!("forward_{name}"));

                        // Add optional material buffer
                        if let Some(mat_buf) = material_buffer {
                            builder = builder.material_buffer(mat_buf);
                        }

                        // Add optional texture
                        if let Some(tex) = texture {
                            builder = builder.texture(tex);
                        }

                        let forward_pass = builder.build(&mut graph)?;

                        log::debug!("Added forward rendering pass for mesh '{name}' with {vertex_count} vertices (PassId: {:?})", forward_pass.pass_id());
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
        self.default_texture = None;
        self.default_material_buffer = None;
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
            normal: [0.0, 1.0, 0.0],
            uv: [0.5, 0.5],
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
            normal: [0.0, 0.0, 1.0],
            uv: [0.0, 0.0],
        };

        let backend_vertex = ForwardPipeline::convert_vertex(&vertex);
        assert_eq!(backend_vertex.normal, [0.0, 0.0, 1.0]);
        assert_eq!(backend_vertex.uv, [0.0, 0.0]);
    }
}

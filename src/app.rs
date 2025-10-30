//! Application framework
//!
//! This module provides the main application structure, event loop,
//! and window management using winit.

use crate::backends::{self, BackendType, GraphicsBackend};
use crate::config::{Backend, Config};
use crate::passes::{ForwardSimplePass, TrianglePass};
use crate::render_graph::{
    Extent3D, ExtentMode, Format, ImageUsageFlags, RenderGraph, ResourceDescriptor, SampleCount,
};
use crate::scene::{GeometryData, Scene, SceneLoader, SceneObject};
use anyhow::{Context, Result};
use std::path::PathBuf;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Main application structure
pub struct App {
    window: Option<Window>,
    backend: Option<Box<dyn GraphicsBackend>>,
    config: Config,
    frame_count: u64,
    render_graph: Option<RenderGraph>,
    scene: Option<Scene>,
}

impl App {
    /// Create a new application instance with the given configuration
    pub fn new(config: Config) -> Result<Self> {
        log::info!("Creating application");
        log::info!("Backend: {}", config.backend);
        log::info!(
            "Window size: {}x{}",
            config.window_size().0,
            config.window_size().1
        );
        log::info!("Debug mode: {}", config.debug);
        log::info!("VSync: {}", config.vsync);
        log::info!("Headless: {}", config.headless);
        if let Some(ref path) = config.screenshot {
            log::info!("Screenshot: {}", path.display());
        }

        // Create backend based on config
        let backend_type = match config.backend {
            Backend::Vulkan => BackendType::Vulkan,
            #[cfg(target_os = "windows")]
            Backend::DirectX => BackendType::DirectX12,
        };

        let mut backend = backends::create_backend(backend_type, config.debug)
            .with_context(|| format!("Failed to create {backend_type} backend"))?;

        log::info!("Successfully created {} backend", backend.backend_type());

        // If headless, initialize immediately
        if config.headless {
            log::info!("Initializing backend in headless mode");
            backend
                .initialize_headless(config.width, config.height)
                .context("Failed to initialize headless mode")?;
            log::info!("Backend initialized in headless mode");
        }

        Ok(Self {
            window: None,
            backend: Some(backend),
            config,
            frame_count: 0,
            render_graph: None,
            scene: None,
        })
    }

    /// Load the scene based on config
    fn load_scene(&mut self) -> Result<()> {
        let scene_path = PathBuf::from(format!("scenes/{}.toml", self.config.scene));
        log::info!("Loading scene: {}", scene_path.display());

        let scene = SceneLoader::load_from_file_static(&scene_path)
            .with_context(|| format!("Failed to load scene from {}", scene_path.display()))?;

        log::info!("Scene loaded: {}", scene.metadata.name);
        log::info!("Objects: {}", scene.objects.len());

        self.scene = Some(scene);
        Ok(())
    }

    /// Build the render graph based on loaded scene
    fn build_render_graph(&mut self) -> Result<()> {
        let scene = self
            .scene
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Scene not loaded"))?;

        let (width, height) = self.config.window_size();
        let mut graph = RenderGraph::new();

        // Check if we have a simple "triangle" scene (debug mode)
        let is_triangle_scene = self.config.scene == "triangle" || scene.objects.is_empty();

        if is_triangle_scene {
            log::info!(
                "Using simple triangle pass for scene: {}",
                self.config.scene
            );

            // Register triangle shaders
            use crate::render_graph::{ShaderDescriptor, ShaderStage};
            graph.register_shader(
                "triangle.vert",
                ShaderDescriptor::from_file("shaders/hlsl/triangle.hlsl", ShaderStage::Vertex)
                    .with_entry_point("VSMain"),
            );
            graph.register_shader(
                "triangle.frag",
                ShaderDescriptor::from_file("shaders/hlsl/triangle.hlsl", ShaderStage::Fragment)
                    .with_entry_point("PSMain"),
            );

            // Create color buffer resource (represents swapchain image)
            let color_desc = ResourceDescriptor::Image {
                format: Format::Bgra8Unorm,
                extent: ExtentMode::Absolute(Extent3D::new_2d(width, height)),
                usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
                samples: SampleCount::One,
                mip_levels: 1,
            };
            let color_buffer = graph.create_resource("swapchain_image", color_desc);

            // Create triangle rendering pass
            TrianglePass::new(&mut graph, color_buffer);
        } else {
            log::info!(
                "Using forward rendering pass for scene: {}",
                scene.metadata.name
            );

            // Register forward shaders
            use crate::render_graph::{ShaderDescriptor, ShaderStage};
            graph.register_shader(
                "forward.vert",
                ShaderDescriptor::from_file("shaders/hlsl/forward.hlsl", ShaderStage::Vertex)
                    .with_entry_point("VSMain"),
            );
            graph.register_shader(
                "forward.frag",
                ShaderDescriptor::from_file("shaders/hlsl/forward.hlsl", ShaderStage::Fragment)
                    .with_entry_point("PSMain"),
            );

            // Create color buffer
            let color_desc = ResourceDescriptor::Image {
                format: Format::Bgra8Unorm,
                extent: ExtentMode::Absolute(Extent3D::new_2d(width, height)),
                usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
                samples: SampleCount::One,
                mip_levels: 1,
            };
            let color_buffer = graph.create_resource("swapchain_image", color_desc);

            // Create depth buffer
            let depth_desc = ResourceDescriptor::Image {
                format: Format::Depth32Float,
                extent: ExtentMode::Absolute(Extent3D::new_2d(width, height)),
                usage: ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
                samples: SampleCount::One,
                mip_levels: 1,
            };
            let depth_buffer = graph.create_resource("depth_buffer", depth_desc);

            // Build vertex and index data from scene
            let mut all_vertices = Vec::new();
            let mut all_indices = Vec::new();
            let mut total_vertices = 0u32;

            for obj in &scene.objects {
                match obj {
                    SceneObject::Mesh { geometry, .. } => {
                        match geometry {
                            GeometryData::Inline { vertices, indices } => {
                                all_vertices.extend_from_slice(vertices);

                                if let Some(idx) = indices {
                                    // Offset indices by current vertex count
                                    all_indices.extend(idx.iter().map(|i| i + total_vertices));
                                } else {
                                    // Generate sequential indices
                                    all_indices.extend(
                                        (0..vertices.len() as u32).map(|i| i + total_vertices),
                                    );
                                }

                                total_vertices += vertices.len() as u32;
                            }
                            GeometryData::File { .. } => {
                                log::warn!("External geometry files not yet supported");
                            }
                        }
                    }
                    SceneObject::GltfModel { .. } => {
                        log::warn!("glTF models not yet supported in render graph");
                    }
                }
            }

            let index_count = all_indices.len() as u32;
            log::info!(
                "Total vertices: {}, indices: {}",
                total_vertices,
                index_count
            );

            // Create vertex buffer
            use crate::render_graph::BufferUsageFlags;
            let vertex_data: Vec<u8> = all_vertices
                .iter()
                .flat_map(|v| {
                    let mut data = Vec::new();
                    data.extend_from_slice(bytemuck::bytes_of(&v.position));
                    data.extend_from_slice(bytemuck::bytes_of(&v.color));
                    data.extend_from_slice(bytemuck::bytes_of(&v.normal));
                    data.extend_from_slice(bytemuck::bytes_of(&v.uv));
                    data
                })
                .collect();

            let vertex_buffer_desc = ResourceDescriptor::Buffer {
                size: vertex_data.len(),
                usage: BufferUsageFlags::new(BufferUsageFlags::VERTEX),
            };
            let vertex_buffer = graph.create_resource("vertex_buffer", vertex_buffer_desc);

            // Create index buffer
            let index_data: Vec<u8> = all_indices.iter().flat_map(|i| i.to_le_bytes()).collect();

            let index_buffer_desc = ResourceDescriptor::Buffer {
                size: index_data.len(),
                usage: BufferUsageFlags::new(BufferUsageFlags::INDEX),
            };
            let _index_buffer = graph.create_resource("index_buffer", index_buffer_desc);

            // Create camera uniforms
            let aspect = width as f32 / height as f32;
            let view = self.calculate_view_matrix(&scene.camera);
            let proj = self.calculate_projection_matrix(&scene.camera, aspect);

            #[repr(C)]
            #[derive(Clone, Copy)]
            struct CameraUniforms {
                view: [[f32; 4]; 4],
                proj: [[f32; 4]; 4],
                view_pos: [f32; 3],
                _padding: f32,
            }
            unsafe impl bytemuck::Pod for CameraUniforms {}
            unsafe impl bytemuck::Zeroable for CameraUniforms {}

            let _camera_uniforms = CameraUniforms {
                view,
                proj,
                view_pos: scene.camera.position(),
                _padding: 0.0,
            };

            let camera_buffer_desc = ResourceDescriptor::Buffer {
                size: std::mem::size_of::<CameraUniforms>(),
                usage: BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
            };
            let camera_buffer = graph.create_resource("camera_uniforms", camera_buffer_desc);

            // Create lighting uniforms
            let lighting = scene.lighting.as_ref().cloned().unwrap_or_default();

            // TODO: This is temporarily hardcoded - should come from scene
            #[repr(C)]
            #[derive(Clone, Copy)]
            struct LightingUniforms {
                ambient: [f32; 3],
                _padding1: f32,
                light_dir: [f32; 3],
                _padding2: f32,
                light_color: [f32; 3],
                light_intensity: f32,
            }
            unsafe impl bytemuck::Pod for LightingUniforms {}
            unsafe impl bytemuck::Zeroable for LightingUniforms {}

            let _lighting_uniforms = LightingUniforms {
                ambient: lighting.ambient,
                _padding1: 0.0,
                light_dir: [-0.5, -1.0, -0.3], // TODO: Get from scene lights
                _padding2: 0.0,
                light_color: [1.0, 1.0, 1.0],
                light_intensity: 1.0,
            };

            let lighting_buffer_desc = ResourceDescriptor::Buffer {
                size: std::mem::size_of::<LightingUniforms>(),
                usage: BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
            };
            let lighting_buffer = graph.create_resource("lighting_uniforms", lighting_buffer_desc);

            // Get transform from first object (for now)
            let transform = scene
                .objects
                .first()
                .and_then(|obj| match obj {
                    SceneObject::Mesh { transform, .. } => Some(*transform),
                    _ => None,
                })
                .unwrap_or_default();

            // Create forward pass
            ForwardSimplePass::builder()
                .color_output(color_buffer)
                .depth_output(depth_buffer)
                .vertex_buffer(vertex_buffer)
                .camera_buffer(camera_buffer)
                .lighting_buffer(lighting_buffer)
                .transform(transform)
                .vertex_count(index_count)
                .with_name("forward_simple")
                .build(&mut graph)?;
        }

        self.render_graph = Some(graph);
        log::info!("Render graph built successfully");
        Ok(())
    }

    /// Calculate view matrix from camera
    fn calculate_view_matrix(&self, camera: &crate::scene::Camera) -> [[f32; 4]; 4] {
        use crate::scene::Camera;

        match camera {
            Camera::Perspective {
                position,
                target,
                up,
                ..
            } => Self::look_at(position, target, up),
            Camera::FreeFly {
                position,
                yaw,
                pitch,
                ..
            } => {
                use std::f32::consts::PI;

                let yaw_rad = yaw * PI / 180.0;
                let pitch_rad = pitch * PI / 180.0;

                let target = [
                    position[0] + yaw_rad.cos() * pitch_rad.cos(),
                    position[1] + pitch_rad.sin(),
                    position[2] + yaw_rad.sin() * pitch_rad.cos(),
                ];

                Self::look_at(position, &target, &[0.0, 1.0, 0.0])
            }
        }
    }

    /// Calculate projection matrix from camera
    fn calculate_projection_matrix(
        &self,
        camera: &crate::scene::Camera,
        aspect: f32,
    ) -> [[f32; 4]; 4] {
        use std::f32::consts::PI;

        let fov = camera.fov() * PI / 180.0;
        let near = camera.near();
        let far = camera.far();

        Self::perspective(fov, aspect, near, far)
    }

    /// Create a look-at view matrix
    fn look_at(eye: &[f32; 3], target: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
        // Forward vector (normalized)
        let f = {
            let dx = target[0] - eye[0];
            let dy = target[1] - eye[1];
            let dz = target[2] - eye[2];
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            [dx / len, dy / len, dz / len]
        };

        // Right vector (normalized)
        let r = {
            let x = up[1] * f[2] - up[2] * f[1];
            let y = up[2] * f[0] - up[0] * f[2];
            let z = up[0] * f[1] - up[1] * f[0];
            let len = (x * x + y * y + z * z).sqrt();
            [x / len, y / len, z / len]
        };

        // Up vector (normalized)
        let u = [
            f[1] * r[2] - f[2] * r[1],
            f[2] * r[0] - f[0] * r[2],
            f[0] * r[1] - f[1] * r[0],
        ];

        [
            [r[0], u[0], -f[0], 0.0],
            [r[1], u[1], -f[1], 0.0],
            [r[2], u[2], -f[2], 0.0],
            [
                -(r[0] * eye[0] + r[1] * eye[1] + r[2] * eye[2]),
                -(u[0] * eye[0] + u[1] * eye[1] + u[2] * eye[2]),
                f[0] * eye[0] + f[1] * eye[1] + f[2] * eye[2],
                1.0,
            ],
        ]
    }

    /// Create a perspective projection matrix
    fn perspective(fov: f32, aspect: f32, near: f32, far: f32) -> [[f32; 4]; 4] {
        let tan_half_fov = (fov / 2.0).tan();

        [
            [1.0 / (aspect * tan_half_fov), 0.0, 0.0, 0.0],
            [0.0, 1.0 / tan_half_fov, 0.0, 0.0],
            [0.0, 0.0, far / (near - far), -1.0],
            [0.0, 0.0, (near * far) / (near - far), 0.0],
        ]
    }

    /// Build the render graph for triangle rendering
    /// Run the application in headless mode (no window)
    pub fn run_headless(&mut self) -> Result<()> {
        log::info!("Running in headless mode");

        let max_frames = self.config.max_frames.unwrap_or(10);
        log::info!("Will render {max_frames} frames");

        // Load scene
        self.load_scene()?;

        // Build render graph
        self.build_render_graph()?;

        // Determine screenshot mode
        let screenshot_interval = self.config.screenshot_interval;
        let capture_sequence = screenshot_interval > 0 && self.config.screenshot.is_some();

        if capture_sequence {
            log::info!("Will capture screenshots every {screenshot_interval} frames");
        }

        while self.frame_count < max_frames {
            if let Some(backend) = &mut self.backend {
                // Compile and execute render graph
                let mut graph = self.render_graph.take().unwrap();
                let compiled = graph.compile()?;

                backend.begin_frame()?;
                backend.execute_graph(&graph, &compiled)?;
                backend.end_frame()?;

                // Put graph back
                self.render_graph = Some(graph);
                self.frame_count += 1;

                if self.frame_count.is_multiple_of(100) {
                    log::debug!("Rendered {} frames", self.frame_count);
                }

                // Capture screenshot at interval if requested
                if capture_sequence && self.frame_count.is_multiple_of(screenshot_interval) {
                    self.capture_screenshot_frame(self.frame_count)?;
                }
            }
        }

        log::info!("Rendered {} frames", self.frame_count);

        // Capture final screenshot if not capturing sequence
        if !capture_sequence && self.config.screenshot.is_some() {
            self.capture_screenshot_frame(self.frame_count)?;
        }

        // Cleanup
        if let Some(backend) = &mut self.backend {
            backend.cleanup();
        }

        Ok(())
    }

    /// Capture and save a screenshot for a specific frame
    fn capture_screenshot_frame(&mut self, frame_number: u64) -> Result<()> {
        if let (Some(ref base_path), Some(backend)) = (&self.config.screenshot, &mut self.backend) {
            // Generate filename with frame number
            let path = if self.config.screenshot_interval > 0 {
                // For sequences, add frame number to filename
                let parent = base_path.parent();
                let stem = base_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("screenshot");
                let ext = base_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("png");

                let filename = format!("{stem}-{frame_number:04}.{ext}");

                if let Some(p) = parent {
                    p.join(filename)
                } else {
                    std::path::PathBuf::from(filename)
                }
            } else {
                // For single screenshot, use original path
                base_path.clone()
            };

            log::info!("Capturing screenshot to {}", path.display());

            let (width, height, pixels) = backend.capture_frame()?;

            // Save as PNG using image crate
            use image::ImageBuffer;
            let img = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
                .context("Failed to create image from captured pixels")?;

            // Create parent directory if it doesn't exist
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory {}", parent.display()))?;
            }

            img.save(&path)
                .with_context(|| format!("Failed to save screenshot to {}", path.display()))?;

            log::info!("Screenshot saved: {width}x{height} -> {}", path.display());
        }

        Ok(())
    }

    /// Run the application event loop
    pub fn run(config: Config) -> Result<()> {
        log::info!("Starting Rusty Renderer");

        // If headless, run without event loop
        if config.headless {
            let mut app = App::new(config)?;
            return app.run_headless();
        }

        // Otherwise, run with event loop
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut app = App::new(config)?;
        event_loop.run_app(&mut app)?;

        Ok(())
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Creating window");

            let window_attributes = Window::default_attributes()
                .with_title("Rusty Renderer")
                .with_inner_size(winit::dpi::LogicalSize::new(
                    self.config.width,
                    self.config.height,
                ));

            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    log::info!(
                        "Window created: {}x{}",
                        window.inner_size().width,
                        window.inner_size().height
                    );

                    // Initialize backend with the window
                    if let Some(backend) = &mut self.backend {
                        if let Err(e) = backend.initialize(&window) {
                            log::error!("Failed to initialize backend: {e}");
                            event_loop.exit();
                            return;
                        }
                        log::info!("Backend initialized successfully");
                    }

                    // Load scene
                    if let Err(e) = self.load_scene() {
                        log::error!("Failed to load scene: {e}");
                        event_loop.exit();
                        return;
                    }

                    // Build render graph
                    if let Err(e) = self.build_render_graph() {
                        log::error!("Failed to build render graph: {e}");
                        event_loop.exit();
                        return;
                    }

                    // Request initial redraw
                    window.request_redraw();

                    self.window = Some(window);
                }
                Err(e) => {
                    log::error!("Failed to create window: {e}");
                    event_loop.exit();
                }
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close requested, shutting down");

                // Capture screenshot if requested
                if let Some(ref path) = self.config.screenshot {
                    log::info!("Capturing screenshot to {}", path.display());
                    if let Some(backend) = &mut self.backend {
                        match backend.capture_frame() {
                            Ok((width, height, pixels)) => {
                                use image::ImageBuffer;
                                match ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                                    width, height, pixels,
                                ) {
                                    Some(img) => {
                                        if let Err(e) = img.save(path) {
                                            log::error!("Failed to save screenshot: {e}");
                                        } else {
                                            log::info!("Screenshot saved: {width}x{height}");
                                        }
                                    }
                                    None => {
                                        log::error!("Failed to create image from captured pixels");
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to capture frame: {e}");
                            }
                        }
                    }
                }

                if let Some(backend) = &mut self.backend {
                    backend.cleanup();
                }
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                log::debug!("Window resized to {}x{}", size.width, size.height);
                if let Some(backend) = &mut self.backend {
                    if let Err(e) = backend.resize(size.width, size.height) {
                        log::error!("Backend resize failed: {e}");
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                // Render a frame using render graph
                if let Some(backend) = &mut self.backend {
                    // Compile graph (could be cached in future)
                    let mut graph = self.render_graph.take().unwrap();
                    let compiled = match graph.compile() {
                        Ok(c) => c,
                        Err(e) => {
                            log::error!("Failed to compile render graph: {e}");
                            self.render_graph = Some(graph);
                            return;
                        }
                    };

                    if let Err(e) = backend.begin_frame() {
                        log::error!("Failed to begin frame: {e}");
                        self.render_graph = Some(graph);
                        return;
                    }

                    if let Err(e) = backend.execute_graph(&graph, &compiled) {
                        log::error!("Failed to execute render graph: {e}");
                        self.render_graph = Some(graph);
                        return;
                    }

                    if let Err(e) = backend.end_frame() {
                        log::error!("Failed to end frame: {e}");
                        self.render_graph = Some(graph);
                        return;
                    }

                    self.render_graph = Some(graph);
                }

                self.frame_count += 1;

                // Check if we've reached max frames
                if let Some(max_frames) = self.config.max_frames {
                    if self.frame_count >= max_frames {
                        log::info!("Rendered {} frames, exiting", self.frame_count);

                        // Capture screenshot if requested
                        if let Some(ref path) = self.config.screenshot {
                            log::info!("Capturing screenshot to {}", path.display());
                            if let Some(backend) = &mut self.backend {
                                match backend.capture_frame() {
                                    Ok((width, height, pixels)) => {
                                        use image::ImageBuffer;
                                        match ImageBuffer::<image::Rgba<u8>, _>::from_raw(
                                            width, height, pixels,
                                        ) {
                                            Some(img) => {
                                                if let Err(e) = img.save(path) {
                                                    log::error!("Failed to save screenshot: {e}");
                                                } else {
                                                    log::info!(
                                                        "Screenshot saved: {width}x{height}"
                                                    );
                                                }
                                            }
                                            None => {
                                                log::error!(
                                                    "Failed to create image from captured pixels"
                                                );
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to capture frame: {e}");
                                    }
                                }
                            }
                        }

                        if let Some(backend) = &mut self.backend {
                            backend.cleanup();
                        }
                        event_loop.exit();
                        return;
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

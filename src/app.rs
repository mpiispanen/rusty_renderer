//! Application framework
//!
//! This module provides the main application structure, event loop,
//! and window management using winit.

use crate::backends::{self, BackendType, GraphicsBackend};
use crate::camera::{self, CameraBackend, CameraController};
use crate::config::{Backend, Config};
use crate::passes::ForwardSimplePass;
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

        // Set camera backend to match graphics backend
        let camera_backend = match config.backend {
            Backend::Vulkan => CameraBackend::Vulkan,
            #[cfg(target_os = "windows")]
            Backend::DirectX => CameraBackend::DirectX,
        };
        camera::set_camera_backend(camera_backend);
        log::info!("Camera backend set to: {:?}", camera_backend);

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
        // If scene already has a path separator or .toml extension, use it as is
        let scene_path = if self.config.scene.contains('/') || self.config.scene.ends_with(".toml")
        {
            PathBuf::from(&self.config.scene)
        } else {
            PathBuf::from(format!("scenes/{}.toml", self.config.scene))
        };
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

        log::info!(
            "Using forward rendering pass for scene: {}",
            scene.metadata.name
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

        // Build vertex data from scene (expanding indices into vertices)
        let mut all_vertices = Vec::new();
        let mut indexed_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut total_vertices = 0u32;

        for obj in &scene.objects {
            match obj {
                SceneObject::Mesh { geometry, .. } => {
                    match geometry {
                        GeometryData::Inline { vertices, indices } => {
                            indexed_vertices.extend_from_slice(vertices);

                            if let Some(idx) = indices {
                                // Offset indices by current vertex count
                                all_indices.extend(idx.iter().map(|i| i + total_vertices));
                            } else {
                                // Generate sequential indices
                                all_indices
                                    .extend((0..vertices.len() as u32).map(|i| i + total_vertices));
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

        // Expand indexed vertices into linear vertex array for now
        // TODO: Use index buffer for efficiency
        for &index in &all_indices {
            all_vertices.push(indexed_vertices[index as usize]);
        }

        let vertex_count = all_vertices.len() as u32;
        log::info!(
            "Total indexed vertices: {}, indices: {}, expanded to: {} vertices",
            total_vertices,
            all_indices.len(),
            vertex_count
        );

        // Log first few vertices for debugging
        for (i, v) in all_vertices.iter().take(3).enumerate() {
            log::info!(
                "  Vertex {}: pos={:?}, normal={:?}, color={:?}",
                i,
                v.position,
                v.normal,
                v.color
            );
        }

        // Create vertex buffer with data via render graph
        use crate::render_graph::BufferUsageFlags;
        // Prepare vertex data matching shader layout: position (3), normal (3), uv (2), color (4)
        let vertex_data: Vec<u8> = all_vertices
            .iter()
            .flat_map(|v| {
                let mut data = Vec::new();
                data.extend_from_slice(bytemuck::bytes_of(&v.position)); // 12 bytes
                data.extend_from_slice(bytemuck::bytes_of(&v.normal)); // 12 bytes
                data.extend_from_slice(bytemuck::bytes_of(&v.uv)); // 8 bytes
                                                                   // Extend color from 3 to 4 components (RGB -> RGBA)
                data.extend_from_slice(bytemuck::bytes_of(&v.color)); // 12 bytes
                data.extend_from_slice(bytemuck::bytes_of(&1.0f32)); // 4 bytes (alpha)
                data
            })
            .collect();

        // Declare vertex buffer with initial data - render graph will allocate and upload
        let vertex_buffer = graph.declare_buffer_with_data(
            "vertex_buffer",
            vertex_data,
            BufferUsageFlags::new(BufferUsageFlags::VERTEX),
        );

        // Create camera controller and get uniforms
        let aspect = width as f32 / height as f32;
        let camera_ctrl = CameraController::from_scene_camera(&scene.camera, width, height);
        let camera_uniforms_glam = camera_ctrl.uniforms();

        log::info!("Camera setup:");
        log::info!("  Position: {:?}", scene.camera.position());
        log::info!("  Target: {:?}", scene.camera.target());
        log::info!("  FOV: {} degrees", scene.camera.fov());
        log::info!(
            "  Near/Far: {} / {}",
            scene.camera.near(),
            scene.camera.far()
        );
        log::info!("  Aspect: {}", aspect);
        log::info!("  ViewProj matrix (from CameraController):");
        for (i, row) in camera_uniforms_glam.view_proj.iter().enumerate() {
            log::info!(
                "    Row {}: [{:.4}, {:.4}, {:.4}, {:.4}]",
                i,
                row[0],
                row[1],
                row[2],
                row[3]
            );
        }

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct CameraUniforms {
            view_proj: [[f32; 4]; 4],
        }
        unsafe impl bytemuck::Pod for CameraUniforms {}
        unsafe impl bytemuck::Zeroable for CameraUniforms {}

        let camera_uniforms = CameraUniforms {
            view_proj: camera_uniforms_glam.view_proj,
        };

        // Declare camera buffer with initial data - render graph will allocate and upload
        let camera_buffer = graph.declare_buffer_with_data(
            "camera_uniforms",
            bytemuck::bytes_of(&camera_uniforms).to_vec(),
            BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
        );

        // Create lighting uniforms
        let lighting = scene.lighting.as_ref().cloned().unwrap_or_default();

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct Light {
            light_type: u32,
            _padding1: u32,
            _padding2: u32,
            _padding3: u32,
            position_or_direction: [f32; 4],
            color_intensity: [f32; 4],
        }
        unsafe impl bytemuck::Pod for Light {}
        unsafe impl bytemuck::Zeroable for Light {}

        const MAX_LIGHTS: usize = 8;

        #[repr(C)]
        #[derive(Clone, Copy)]
        struct LightingUniforms {
            ambient_light_count: [f32; 4], // RGB ambient + light count
            lights: [Light; MAX_LIGHTS],
        }
        unsafe impl bytemuck::Pod for LightingUniforms {}
        unsafe impl bytemuck::Zeroable for LightingUniforms {}

        // Build lights array
        let mut lights_array = [Light {
            light_type: 0,
            _padding1: 0,
            _padding2: 0,
            _padding3: 0,
            position_or_direction: [0.0; 4],
            color_intensity: [0.0; 4],
        }; MAX_LIGHTS];

        let light_count = lighting.lights.len().min(MAX_LIGHTS);
        for (i, scene_light) in lighting.lights.iter().take(MAX_LIGHTS).enumerate() {
            lights_array[i] = match scene_light {
                crate::scene::Light::Directional {
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
                crate::scene::Light::Point {
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

        let lighting_uniforms = LightingUniforms {
            ambient_light_count: [
                lighting.ambient[0],
                lighting.ambient[1],
                lighting.ambient[2],
                light_count as f32,
            ],
            lights: lights_array,
        };

        // Declare lighting buffer with initial data - render graph will allocate and upload
        let lighting_buffer = graph.declare_buffer_with_data(
            "lighting_uniforms",
            bytemuck::bytes_of(&lighting_uniforms).to_vec(),
            BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
        );

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
            .vertex_count(vertex_count)
            .with_name("forward_simple")
            .build(&mut graph)?;
        self.render_graph = Some(graph);
        log::info!("Render graph built successfully");
        Ok(())
    }

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
                log::debug!("After execute_graph, backend still exists");
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

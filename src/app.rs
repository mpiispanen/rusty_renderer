//! Application framework
//!
//! This module provides the main application structure, event loop,
//! and window management using winit.

use crate::backends::{self, BackendType, GraphicsBackend};
use crate::camera::{self, CameraBackend, CameraController};
use crate::config::{Backend, Config};
use crate::passes::{ForwardSimplePass, ForwardSimpleSceneResources, ShadowMapPass};
use crate::render_graph::{
    Extent3D, ExtentMode, Format, ImageUsageFlags, RenderGraph, ResourceDescriptor, ResourceId,
    SampleCount,
};
use crate::scene::{Scene, SceneLoader};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
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
    camera: Option<CameraController>,
    camera_buffer: Option<ResourceId>,
    input_state: InputState,
    last_frame_time: Instant,
    mouse_captured: bool,
}

/// Input state tracking
#[derive(Default)]
struct InputState {
    keys_pressed: HashSet<KeyCode>,
    mouse_delta: (f64, f64),
    last_mouse_pos: Option<(f64, f64)>,
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
            camera: None,
            camera_buffer: None,
            input_state: InputState::default(),
            last_frame_time: Instant::now(),
            mouse_captured: false,
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

        let ForwardSimpleSceneResources {
            vertex_buffer,
            index_buffer,
            vertex_count,
            index_count,
            camera_buffer,
            lighting_buffer,
            transform,
        } = ForwardSimplePass::prepare_scene_resources(scene, &mut graph, width, height)?;

        // Create camera controller from scene camera
        let camera_ctrl = CameraController::from_scene_camera(&scene.camera, width, height);
        self.camera = Some(camera_ctrl);
        self.camera_buffer = Some(camera_buffer);

        log::info!("ForwardSimplePass prepared {vertex_count} vertices");

        // Check if scene has directional light for shadow mapping
        let has_directional_light = scene
            .lighting
            .as_ref()
            .and_then(|l| l.lights.first())
            .map(|l| matches!(l.light_type(), crate::scene::LightType::Directional))
            .unwrap_or(false);

        let mut shadow_map = None;
        let mut shadow_uniforms = None;

        if has_directional_light {
            log::info!("Scene has directional light - enabling shadow mapping");
            
            // Get light direction from scene
            let light_direction = scene
                .lighting
                .as_ref()
                .and_then(|l| l.lights.first())
                .and_then(|l| l.direction())
                .map(|d| glam::Vec3::from_array(d))
                .unwrap_or(glam::Vec3::new(0.0, -1.0, 0.0));

            // Prepare shadow map resources
            let shadow_resources =
                ShadowMapPass::prepare_resources(&mut graph, light_direction, 1024);

            shadow_map = Some(shadow_resources.shadow_map);
            shadow_uniforms = Some(shadow_resources.light_uniforms);

            // Add shadow map pass (runs before forward pass)
            ShadowMapPass::builder()
                .shadow_map_output(shadow_resources.shadow_map)
                .vertex_buffer(vertex_buffer)
                .index_buffer(index_buffer)
                .light_uniforms(shadow_resources.light_uniforms)
                .index_count(index_count)
                .with_name("shadow_map")
                .build(&mut graph)?;

            log::info!("Shadow map pass added to render graph");
        }

        // Create forward pass
        let mut forward_builder = ForwardSimplePass::builder()
            .color_output(color_buffer)
            .depth_output(depth_buffer)
            .vertex_buffer(vertex_buffer)
            .index_buffer(index_buffer)
            // Camera is now passed via push constants, not uniform buffer
            .lighting_buffer(lighting_buffer)
            .transform(transform)
            .vertex_count(vertex_count)
            .index_count(index_count)
            .with_name("forward_simple");

        // Add shadow resources if available
        if let Some(sm) = shadow_map {
            forward_builder = forward_builder.shadow_map(sm);
        }
        if let Some(su) = shadow_uniforms {
            forward_builder = forward_builder.shadow_uniforms(su);
        }

        forward_builder.build(&mut graph)?;
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
            // Update camera uniforms for this frame (even though camera is static in headless mode)
            if let Some(camera) = &self.camera {
                camera::set_current_camera_uniforms(camera.uniforms());
            }
            
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
            use image::{ImageBuffer, imageops};
            let img = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
                .context("Failed to create image from captured pixels")?;
            
            // Flip vertically - Vulkan framebuffer has (0,0) at top-left
            // but the rendered scene is upside down
            let img = imageops::flip_vertical(&img);

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

    /// Update camera based on input state
    fn update_camera(&mut self, delta_time: f32) {
        if let Some(camera) = &mut self.camera {
            // Movement speed (units per second)
            let base_speed = 5.0;
            let speed = if self.input_state.keys_pressed.contains(&KeyCode::ShiftLeft) {
                base_speed * 2.0
            } else {
                base_speed
            };

            let move_distance = speed * delta_time;

            // WASD movement
            if self.input_state.keys_pressed.contains(&KeyCode::KeyW) {
                camera.move_forward(move_distance);
            }
            if self.input_state.keys_pressed.contains(&KeyCode::KeyS) {
                camera.move_forward(-move_distance);
            }
            if self.input_state.keys_pressed.contains(&KeyCode::KeyA) {
                camera.move_right(move_distance);
            }
            if self.input_state.keys_pressed.contains(&KeyCode::KeyD) {
                camera.move_right(-move_distance);
            }

            // QE for up/down
            if self.input_state.keys_pressed.contains(&KeyCode::KeyE) {
                camera.move_up(move_distance);
            }
            if self.input_state.keys_pressed.contains(&KeyCode::KeyQ) {
                camera.move_up(-move_distance);
            }

            // Mouse look (only if mouse is captured)
            if self.mouse_captured && self.input_state.mouse_delta != (0.0, 0.0) {
                let sensitivity = 0.1;
                let delta_yaw = self.input_state.mouse_delta.0 as f32 * sensitivity;
                let delta_pitch = self.input_state.mouse_delta.1 as f32 * sensitivity;
                camera.rotate(delta_yaw, delta_pitch);
            }

            // Reset mouse delta
            self.input_state.mouse_delta = (0.0, 0.0);
        }
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
                // Update camera aspect ratio
                if let Some(camera) = &mut self.camera {
                    camera.set_aspect_ratio(size.width, size.height);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            self.input_state.keys_pressed.insert(keycode);
                            
                            // Escape to exit
                            if keycode == KeyCode::Escape {
                                if let Some(backend) = &mut self.backend {
                                    backend.cleanup();
                                }
                                event_loop.exit();
                                return;
                            }
                        }
                        ElementState::Released => {
                            self.input_state.keys_pressed.remove(&keycode);
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some(last_pos) = self.input_state.last_mouse_pos {
                    let delta = (
                        position.x - last_pos.0,
                        position.y - last_pos.1,
                    );
                    if self.mouse_captured {
                        self.input_state.mouse_delta.0 += delta.0;
                        self.input_state.mouse_delta.1 += delta.1;
                    }
                }
                self.input_state.last_mouse_pos = Some((position.x, position.y));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                // Optional: Click to capture mouse
                if !self.config.headless
                    && state == ElementState::Pressed
                    && button == MouseButton::Left
                    && !self.mouse_captured
                {
                    self.mouse_captured = true;
                    if let Some(window) = &self.window {
                        window.set_cursor_visible(false);
                        let _ = window.set_cursor_grab(
                            winit::window::CursorGrabMode::Confined
                        );
                    }
                    log::info!("Mouse captured");
                }
            }
            WindowEvent::RedrawRequested => {
                // Update camera based on input (tracked but not yet applied to GPU)
                let now = Instant::now();
                let delta_time = (now - self.last_frame_time).as_secs_f32();
                self.last_frame_time = now;
                self.update_camera(delta_time);

                // Update global camera uniforms for this frame (push constant rendering)
                if let Some(camera) = &self.camera {
                    camera::set_current_camera_uniforms(camera.uniforms());
                }

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

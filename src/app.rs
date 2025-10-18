//! Application framework
//!
//! This module provides the main application structure, event loop,
//! and window management using winit.

use crate::backends::{self, BackendType, GraphicsBackend};
use crate::config::{Backend, Config};
use anyhow::{Context, Result};
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
            Backend::Wgpu => BackendType::Wgpu,
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
        })
    }

    /// Run the application in headless mode (no window)
    pub fn run_headless(&mut self) -> Result<()> {
        log::info!("Running in headless mode");

        let max_frames = self.config.max_frames.unwrap_or(10);
        log::info!("Will render {max_frames} frames");

        while self.frame_count < max_frames {
            if let Some(backend) = &mut self.backend {
                backend.begin_frame()?;
                backend.end_frame()?;
                self.frame_count += 1;

                if self.frame_count % 100 == 0 {
                    log::debug!("Rendered {} frames", self.frame_count);
                }
            }
        }

        log::info!("Rendered {} frames", self.frame_count);

        // Capture screenshot if requested
        if let Some(ref path) = self.config.screenshot {
            log::info!("Capturing screenshot to {}", path.display());
            if let Some(backend) = &mut self.backend {
                let (width, height, pixels) = backend.capture_frame()?;

                // Save as PNG using image crate
                use image::ImageBuffer;
                let img = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
                    .context("Failed to create image from captured pixels")?;

                img.save(path)
                    .with_context(|| format!("Failed to save screenshot to {}", path.display()))?;

                log::info!("Screenshot saved: {width}x{height}");
            }
        }

        // Cleanup
        if let Some(backend) = &mut self.backend {
            backend.cleanup();
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
                // Render a frame
                if let Some(backend) = &mut self.backend {
                    if let Err(e) = backend.begin_frame() {
                        log::error!("Failed to begin frame: {e}");
                        return;
                    }

                    if let Err(e) = backend.end_frame() {
                        log::error!("Failed to end frame: {e}");
                        return;
                    }
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

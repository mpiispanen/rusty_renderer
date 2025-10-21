//! Application runner
//!
//! Manages the main application lifecycle: initialization, event loop, and shutdown.

use super::ApplicationArgs;
use crate::pipelines::{PipelineFactory, RenderPipeline};
use crate::scene::{Scene, SceneLoader};
use anyhow::{Context, Result};
use std::path::PathBuf;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

/// Application runner
///
/// Coordinates the main application lifecycle:
/// 1. Parse arguments
/// 2. Load scene
/// 3. Initialize backend
/// 4. Create pipeline
/// 5. Run event loop
/// 6. Cleanup
pub struct ApplicationRunner {
    args: ApplicationArgs,
    scene: Option<Scene>,
    pipeline: Option<Box<dyn RenderPipeline>>,
}

/// Windowed application state for event loop
struct WindowedApp {
    window: Option<Window>,
    backend: Option<Box<dyn crate::backends::GraphicsBackend>>,
    scene: Scene,
    pipeline: Box<dyn RenderPipeline>,
    graph: Option<crate::render_graph::RenderGraph>,
    compiled: Option<crate::render_graph::CompiledGraph>,
    screenshot_path: Option<PathBuf>,
    frame_count: u64,
    max_frames: u32,
}

impl ApplicationRunner {
    /// Create a new application runner
    pub fn new(args: ApplicationArgs) -> Self {
        Self {
            args,
            scene: None,
            pipeline: None,
        }
    }

    /// Create from command line arguments
    pub fn from_args() -> Result<Self> {
        let args = ApplicationArgs::parse_args();
        args.validate()?;
        Ok(Self::new(args))
    }

    /// Run the application
    pub fn run(mut self) -> Result<()> {
        // Handle list commands
        if self.args.list_scenes {
            return self.list_scenes();
        }

        if self.args.list_pipelines {
            return self.list_pipelines();
        }

        // Load scene
        self.load_scene()?;

        // Create pipeline
        self.create_pipeline()?;

        // Initialize and run
        self.initialize_and_run()?;

        Ok(())
    }

    /// List available scenes
    fn list_scenes(&self) -> Result<()> {
        println!("📦 Available scenes:");
        println!();

        let scene_dir = self.args.scene_directory();
        let scenes = SceneLoader::list_scenes(&scene_dir)?;

        if scenes.is_empty() {
            println!("  No scenes found in {}/", scene_dir.display());
            println!();
            println!("  Create a scene file with .toml extension in the scenes/ directory.");
        } else {
            for scene_name in scenes {
                let scene_path = scene_dir.join(format!("{scene_name}.toml"));

                // Try to load and get metadata
                if let Ok(scene) = SceneLoader::load_from_file(&scene_path) {
                    println!("  • {} - {}", scene_name, scene.metadata.name);
                    if !scene.metadata.description.is_empty() {
                        println!("    {}", scene.metadata.description);
                    }
                } else {
                    println!("  • {scene_name} (failed to load)");
                }
            }
        }

        println!();
        Ok(())
    }

    /// List available pipelines
    fn list_pipelines(&self) -> Result<()> {
        println!("🔧 Available pipelines:");
        println!();

        let pipelines = PipelineFactory::list_pipelines();

        for pipeline_name in pipelines {
            // Create pipeline to get info
            if let Ok(pipeline) = PipelineFactory::create(&pipeline_name) {
                println!("  • {} - {}", pipeline_name, pipeline.name());
            }
        }

        println!();
        Ok(())
    }

    /// Load the scene
    fn load_scene(&mut self) -> Result<()> {
        let scene_path = self
            .args
            .scene
            .as_ref()
            .context("No scene specified. Use --scene <file> or --list-scenes")?;

        log::info!("Loading scene from: {}", scene_path.display());

        let scene = SceneLoader::load_from_file(scene_path)
            .with_context(|| format!("Failed to load scene: {}", scene_path.display()))?;

        log::info!("Scene loaded: {}", scene.metadata.name);
        log::info!("  Objects: {}", scene.objects.len());

        self.scene = Some(scene);
        Ok(())
    }

    /// Create the rendering pipeline
    fn create_pipeline(&mut self) -> Result<()> {
        log::info!("Creating pipeline: {}", self.args.pipeline);

        let pipeline = PipelineFactory::create(&self.args.pipeline)
            .with_context(|| format!("Failed to create pipeline: {}", self.args.pipeline))?;

        log::info!("Pipeline created: {}", pipeline.name());

        self.pipeline = Some(pipeline);
        Ok(())
    }

    /// Initialize backend and run the render loop
    fn initialize_and_run(&mut self) -> Result<()> {
        let scene = self.scene.take().context("No scene loaded")?;
        let pipeline = self.pipeline.take().context("No pipeline created")?;

        log::info!("Initializing application...");
        log::info!("  Scene: {}", scene.metadata.name);
        log::info!("  Pipeline: {}", pipeline.name());
        log::info!(
            "  Mode: {}",
            if self.args.headless {
                "headless"
            } else {
                "windowed"
            }
        );

        // Determine backend type
        let backend_type = self.args.backend_type();

        // Create backend
        log::info!("Creating backend: {backend_type}");
        let mut backend = crate::backends::create_backend(backend_type, true)?;

        // Run in appropriate mode
        if self.args.headless {
            // Headless mode: initialize and run without window
            backend.initialize_headless(self.args.width, self.args.height)?;
            log::info!(
                "Backend initialized (headless {}x{})",
                self.args.width,
                self.args.height
            );

            // Setup pipeline
            log::info!("Setting up pipeline...");
            let mut pipeline = pipeline;
            pipeline.setup(&mut *backend)?;

            // Build render graph
            log::info!("Building render graph...");
            let mut graph = pipeline.build_graph(&scene, &mut *backend)?;

            // Compile render graph
            log::info!("Compiling render graph...");
            let compiled = graph.compile()?;
            log::info!(
                "Render graph compiled: {} passes",
                compiled.execution_order.len()
            );

            // Run rendering
            let screenshot = self.args.screenshot.clone();
            let max_frames = self.args.max_frames;
            Self::run_headless_static(&mut *backend, &graph, &compiled, max_frames, screenshot)?;

            // Cleanup - drop graph and compiled first to release buffer references
            drop(compiled);
            drop(graph);
            
            pipeline.cleanup(&mut *backend);
            backend.cleanup();

            log::info!("Application shutdown complete");
        } else {
            // Windowed mode: create event loop and window
            log::info!("Starting windowed mode...");
            let event_loop = EventLoop::new()
                .context("Failed to create event loop")?;
            event_loop.set_control_flow(ControlFlow::Poll);

            let mut app = WindowedApp {
                window: None,
                backend: Some(backend),
                scene,
                pipeline,
                graph: None,
                compiled: None,
                screenshot_path: self.args.screenshot.clone(),
                frame_count: 0,
                max_frames: self.args.max_frames,
            };

            event_loop.run_app(&mut app)
                .context("Event loop error")?;
        }

        Ok(())
    }

    /// Run in headless mode (single frame or limited frames)
    fn run_headless_static(
        backend: &mut dyn crate::backends::GraphicsBackend,
        graph: &crate::render_graph::RenderGraph,
        compiled: &crate::render_graph::CompiledGraph,
        max_frames: u32,
        screenshot: Option<PathBuf>,
    ) -> Result<()> {
        let max_frames = if max_frames > 0 {
            max_frames
        } else {
            1 // Default to single frame in headless
        };

        log::info!("Rendering {max_frames} frame(s)...");

        for frame in 0..max_frames {
            log::debug!("Frame {}/{}", frame + 1, max_frames);

            backend.begin_frame()?;
            backend.execute_graph(graph, compiled)?;
            backend.end_frame()?;
        }

        log::info!("Rendering complete");

        // Capture screenshot if requested
        if let Some(screenshot_path) = screenshot {
            log::info!("Capturing screenshot...");
            let (width, height, pixels) = backend.capture_frame()?;
            log::info!(
                "Frame captured: {}x{} ({} bytes)",
                width,
                height,
                pixels.len()
            );

            image::save_buffer(
                &screenshot_path,
                &pixels,
                width,
                height,
                image::ColorType::Rgba8,
            )
            .with_context(|| format!("Failed to save screenshot: {}", screenshot_path.display()))?;

            log::info!("Screenshot saved to: {}", screenshot_path.display());
        }

        Ok(())
    }
}

impl Drop for ApplicationRunner {
    fn drop(&mut self) {
        log::debug!("Application runner cleanup");
    }
}

impl Drop for WindowedApp {
    fn drop(&mut self) {
        log::info!("Cleaning up windowed application");
        
        // Drop graph and compiled first to release buffer references
        self.compiled = None;
        self.graph = None;
        
        // Cleanup pipeline
        if let Some(backend) = &mut self.backend {
            self.pipeline.cleanup(&mut **backend);
            backend.cleanup();
        }
        
        log::info!("Windowed application cleaned up");
    }
}

impl ApplicationHandler for WindowedApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            log::info!("Creating window: {}", self.scene.metadata.name);

            let window_attributes = Window::default_attributes()
                .with_title(&self.scene.metadata.name)
                .with_inner_size(winit::dpi::LogicalSize::new(800u32, 600u32));

            match event_loop.create_window(window_attributes) {
                Ok(window) => {
                    log::info!(
                        "Window created: {}x{}",
                        window.inner_size().width,
                        window.inner_size().height
                    );

                    // Initialize backend with window
                    if let Some(backend) = &mut self.backend {
                        if let Err(e) = backend.initialize(&window) {
                            log::error!("Failed to initialize backend: {e}");
                            event_loop.exit();
                            return;
                        }
                        log::info!("Backend initialized with window");

                        // Setup pipeline
                        if let Err(e) = self.pipeline.setup(&mut **backend) {
                            log::error!("Failed to setup pipeline: {e}");
                            event_loop.exit();
                            return;
                        }

                        // Build render graph
                        match self.pipeline.build_graph(&self.scene, &mut **backend) {
                            Ok(mut graph) => {
                                log::info!("Render graph built");
                                
                                // Compile graph
                                match graph.compile() {
                                    Ok(compiled) => {
                                        log::info!("Render graph compiled: {} passes", compiled.execution_order.len());
                                        self.compiled = Some(compiled);
                                        self.graph = Some(graph);
                                    }
                                    Err(e) => {
                                        log::error!("Failed to compile render graph: {e}");
                                        event_loop.exit();
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to build render graph: {e}");
                                event_loop.exit();
                                return;
                            }
                        }
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
                if let Some(ref path) = self.screenshot_path {
                    log::info!("Capturing screenshot to {}", path.display());
                    if let Some(backend) = &mut self.backend {
                        match backend.capture_frame() {
                            Ok((width, height, pixels)) => {
                                if let Err(e) = image::save_buffer(
                                    path,
                                    &pixels,
                                    width,
                                    height,
                                    image::ColorType::Rgba8,
                                ) {
                                    log::error!("Failed to save screenshot: {e}");
                                } else {
                                    log::info!("Screenshot saved: {width}x{height}");
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to capture frame: {e}");
                            }
                        }
                    }
                }

                // Cleanup
                if let Some(backend) = &mut self.backend {
                    self.pipeline.cleanup(&mut **backend);
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
                if let (Some(backend), Some(graph), Some(compiled)) = 
                    (&mut self.backend, &self.graph, &self.compiled) {
                    
                    if let Err(e) = backend.begin_frame() {
                        log::error!("Failed to begin frame: {e}");
                        return;
                    }

                    if let Err(e) = backend.execute_graph(graph, compiled) {
                        log::error!("Failed to execute render graph: {e}");
                        return;
                    }

                    if let Err(e) = backend.end_frame() {
                        log::error!("Failed to end frame: {e}");
                        return;
                    }

                    self.frame_count += 1;
                    
                    // Check if we've reached max frames
                    if self.max_frames > 0 && self.frame_count >= self.max_frames as u64 {
                        log::info!("Rendered {} frames, exiting", self.frame_count);
                        event_loop.exit();
                        return;
                    }
                    
                    // Request next frame
                    if let Some(window) = &self.window {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                // Handle keyboard input
                if event.state.is_pressed() {
                    use winit::keyboard::{KeyCode, PhysicalKey};
                    
                    if let PhysicalKey::Code(KeyCode::Escape) = event.physical_key {
                        log::info!("Escape pressed, closing window");
                        event_loop.exit();
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let args = ApplicationArgs {
            scene: None,
            pipeline: "simple".to_string(),
            backend: None,
            headless: false,
            list_scenes: false,
            list_pipelines: false,
            width: 800,
            height: 600,
            max_frames: 0,
            screenshot: None,
        };

        let runner = ApplicationRunner::new(args);
        assert!(runner.scene.is_none());
        assert!(runner.pipeline.is_none());
    }
}

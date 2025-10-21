//! Application runner
//!
//! Manages the main application lifecycle: initialization, event loop, and shutdown.

use super::ApplicationArgs;
use crate::pipelines::{PipelineFactory, RenderPipeline};
use crate::scene::{Scene, SceneLoader};
use anyhow::{Context, Result};
use std::path::PathBuf;

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
        let scene = self.scene.as_ref().context("No scene loaded")?;
        let pipeline = self.pipeline.as_mut().context("No pipeline created")?;

        log::info!("Initializing application...");
        log::info!("  Scene: {}", scene.metadata.name);
        log::info!("  Pipeline: {}", pipeline.name());
        log::info!(
            "  Mode: {}",
            if self.args.headless {
                "headless"
            } else {
                "interactive"
            }
        );

        // Determine backend type
        let backend_type = self.args.backend_type();

        // Create backend
        log::info!("Creating backend: {}", backend_type);
        let mut backend = crate::backends::create_backend(backend_type, true)?;

        // Initialize backend (headless or windowed)
        if self.args.headless {
            backend.initialize_headless(self.args.width, self.args.height)?;
            log::info!(
                "Backend initialized (headless {}x{})",
                self.args.width,
                self.args.height
            );
        } else {
            // For now, use headless mode even for interactive
            // TODO: Implement proper windowed mode with event loop
            backend.initialize_headless(self.args.width, self.args.height)?;
            log::info!(
                "Backend initialized ({}x{})",
                self.args.width,
                self.args.height
            );
            log::warn!("Note: Windowed mode with event loop coming in future update");
        }

        // Setup pipeline
        log::info!("Setting up pipeline...");
        pipeline.setup(&mut *backend)?;

        // Build render graph
        log::info!("Building render graph...");
        let mut graph = pipeline.build_graph(scene, &mut *backend)?;

        // Compile render graph
        log::info!("Compiling render graph...");
        let compiled = graph.compile()?;
        log::info!("Render graph compiled: {} passes", compiled.execution_order.len());

        // Run rendering
        let screenshot = self.args.screenshot.clone();
        let max_frames = self.args.max_frames;
        if self.args.headless {
            Self::run_headless_static(&mut *backend, &graph, &compiled, max_frames, screenshot)?;
        } else {
            // For now, same as headless
            // TODO: Implement proper event loop
            Self::run_headless_static(&mut *backend, &graph, &compiled, max_frames, screenshot)?;
        }

        // Cleanup
        pipeline.cleanup(&mut *backend);
        backend.cleanup();

        log::info!("Application shutdown complete");

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

        log::info!("Rendering {} frame(s)...", max_frames);

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
            log::info!("Frame captured: {}x{} ({} bytes)", width, height, pixels.len());

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

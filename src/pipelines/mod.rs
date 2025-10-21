//! Render pipeline templates
//!
//! This module provides reusable pipeline templates that define how scenes
//! are rendered. Instead of manually constructing render graphs for each
//! scene, pipelines provide pre-configured rendering strategies.

use crate::render_graph::RenderGraph;
use crate::scene::Scene;
use anyhow::Result;

pub mod forward;
pub mod simple;

pub use forward::ForwardPipeline;
pub use simple::SimplePipeline;

/// Render pipeline trait
///
/// A pipeline defines how to render a scene by constructing the appropriate
/// render graph and managing resources.
pub trait RenderPipeline {
    /// Get the pipeline name
    fn name(&self) -> &str;

    /// Setup the pipeline with a backend
    /// This is called once when the pipeline is first created
    fn setup(&mut self, backend: &mut dyn crate::backends::GraphicsBackend) -> Result<()>;

    /// Build render graph for a scene
    /// This is called each frame to construct the render graph
    fn build_graph(
        &mut self,
        scene: &Scene,
        backend: &mut dyn crate::backends::GraphicsBackend,
    ) -> Result<RenderGraph>;

    /// Cleanup pipeline resources
    fn cleanup(&mut self, backend: &mut dyn crate::backends::GraphicsBackend);
}

/// Pipeline factory for creating pipelines by name
pub struct PipelineFactory;

impl PipelineFactory {
    /// Create a pipeline by name
    pub fn create(name: &str) -> Result<Box<dyn RenderPipeline>> {
        match name.to_lowercase().as_str() {
            "simple" => Ok(Box::new(SimplePipeline::new())),
            "forward" => Ok(Box::new(ForwardPipeline::new())),
            _ => anyhow::bail!("Unknown pipeline: {name}"),
        }
    }

    /// List available pipelines
    pub fn list_pipelines() -> Vec<String> {
        vec!["simple".to_string(), "forward".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_factory() {
        let pipelines = PipelineFactory::list_pipelines();
        assert!(!pipelines.is_empty());
        assert!(pipelines.contains(&"simple".to_string()));
    }

    #[test]
    fn test_create_pipeline() {
        let pipeline = PipelineFactory::create("simple");
        assert!(pipeline.is_ok());

        let pipeline = PipelineFactory::create("invalid");
        assert!(pipeline.is_err());
    }
}

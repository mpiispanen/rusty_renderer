//! Simple rendering pipeline
//!
//! Renders scenes with vertex colors, no lighting. This is the most basic
//! pipeline for testing and simple visualization.

use super::*;
use crate::render_graph::RenderGraph;
use crate::scene::{GeometryData, SceneObject};

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
        // No setup needed for simple pipeline
        log::info!("Simple pipeline setup complete");
        Ok(())
    }

    fn build_graph(&mut self, scene: &Scene, _backend: &mut dyn crate::backends::GraphicsBackend) -> Result<RenderGraph> {
        log::debug!("Building render graph for scene: {}", scene.metadata.name);
        
        // For M10 Phase 0, we're just setting up the structure
        // The actual render graph construction will be done when we integrate
        // with the unified application in the next phase
        
        let graph = RenderGraph::new();

        log::info!(
            "SimplePipeline: Scene '{}' has {} objects",
            scene.metadata.name,
            scene.objects.len()
        );
        
        // Log what we would render
        for obj in &scene.objects {
            match obj {
                SceneObject::Mesh { name, geometry, .. } => {
                    match geometry {
                        GeometryData::Inline { vertices, .. } => {
                            log::info!("  - Mesh '{}': {} vertices", name, vertices.len());
                        }
                        GeometryData::File { path } => {
                            log::info!("  - Mesh '{}': external file {}", name, path);
                        }
                    }
                }
                SceneObject::GltfModel { name, path, .. } => {
                    log::info!("  - glTF Model '{}': {}", name, path);
                }
            }
        }

        // TODO: Actually build the render graph with passes
        // For now, just return an empty graph
        // This will be completed when we integrate with the application

        Ok(graph)
    }

    fn cleanup(&mut self, _backend: &mut dyn crate::backends::GraphicsBackend) {
        log::info!("Simple pipeline cleanup");
        // No cleanup needed
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

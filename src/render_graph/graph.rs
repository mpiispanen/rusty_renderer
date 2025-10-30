//! Render graph core implementation
//!
//! This module implements the main render graph structure and compilation.

use crate::render_graph::barrier::{Barrier, BarrierInserter};
use crate::render_graph::pass::{PassId, PassKind, RenderPass};
use crate::render_graph::pipeline::PipelineBuilder;
use crate::render_graph::resource::{
    BufferUsageFlags, ExtentMode, Format, ImageUsageFlags, Resource, ResourceDescriptor,
    ResourceId, ResourceKind, SampleCount, SamplerDescriptor,
};
use crate::render_graph::shader::{ShaderDescriptor, ShaderHandle, ShaderRegistry};
use std::collections::HashMap;
use thiserror::Error;

/// Render graph errors
#[derive(Debug, Error)]
pub enum RenderGraphError {
    #[error("Cyclic dependency detected in render graph")]
    CyclicDependency,

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Pass not found: {0}")]
    PassNotFound(String),

    #[error("Invalid resource usage: {0}")]
    InvalidUsage(String),

    #[error("Incompatible resource formats")]
    IncompatibleFormats,

    #[error("Resource {0} has no producer")]
    NoProducer(String),

    #[error("Multiple producers for resource {0}")]
    MultipleProducers(String),
}

/// Result type for render graph operations
pub type Result<T> = std::result::Result<T, RenderGraphError>;

/// Dependency edge in the graph
#[derive(Debug, Clone)]
#[allow(dead_code)] // resource field will be used for barrier insertion in M6.3
struct Dependency {
    /// Pass that depends on another
    dependent: PassId,
    /// Pass that is depended on
    dependency: PassId,
    /// Resource that creates the dependency
    resource: ResourceId,
}

/// Compiled render graph ready for execution
pub struct CompiledGraph {
    /// Execution order of passes
    pub execution_order: Vec<PassId>,
    /// Resource producers (pass that writes to each resource)
    pub producers: HashMap<ResourceId, PassId>,
    /// Barriers to insert between passes
    pub barriers: Vec<Barrier>,
    /// Pipeline descriptions for each graphics/compute pass
    pub pipeline_descriptions: HashMap<PassId, PipelineBuilder>,
    /// Resources that need to be allocated by the backend (excluding external resources)
    pub resources_to_allocate: Vec<ResourceId>,
}

/// Main render graph structure
pub struct RenderGraph {
    /// All passes in the graph
    passes: Vec<RenderPass>,
    /// All resources in the graph
    resources: Vec<Resource>,
    /// Pass lookup by name
    pass_names: HashMap<String, PassId>,
    /// Resource lookup by name
    resource_names: HashMap<String, ResourceId>,
    /// Shader registry
    shader_registry: ShaderRegistry,
    /// Next pass ID
    next_pass_id: usize,
    /// Next resource ID
    next_resource_id: usize,
}

impl RenderGraph {
    /// Create a new empty render graph
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            resources: Vec::new(),
            pass_names: HashMap::new(),
            resource_names: HashMap::new(),
            shader_registry: ShaderRegistry::new(),
            next_pass_id: 0,
            next_resource_id: 0,
        }
    }

    /// Add a new pass to the graph
    pub fn add_pass(&mut self, pass: RenderPass) -> PassId {
        let id = pass.id;
        self.pass_names.insert(pass.name.clone(), id);
        self.passes.push(pass);
        id
    }

    /// Add a declarative pass to the graph
    ///
    /// This is the preferred way to add passes using the new declarative API.
    /// The pass will declare its resources and dependencies, which are then
    /// automatically configured.
    ///
    /// # Example
    /// ```ignore
    /// let pass = MyPass::new(color_buffer);
    /// let pass_id = graph.add_declarative_pass(pass);
    /// ```
    ///
    /// # Arguments
    /// * `pass` - A type implementing DeclarativePass
    ///
    /// # Returns
    /// The PassId of the newly added pass
    pub fn add_declarative_pass<T: crate::render_graph::pass::DeclarativePass + 'static>(
        &mut self,
        pass: T,
    ) -> PassId {
        use crate::render_graph::pass::{DeclarativePassAdapter, PassBuilder};

        let id = PassId(self.next_pass_id);
        self.next_pass_id += 1;

        // Let the pass declare any resources it needs
        pass.declare_resources(self);

        // Build dependencies using the builder
        let mut builder = PassBuilder::new(id);
        pass.declare_dependencies(&mut builder);
        let (inputs, outputs) = builder.build();

        // Create the RenderPass with the adapter
        let mut render_pass = RenderPass::new(id, pass.name(), pass.kind());
        render_pass.inputs = inputs;
        render_pass.outputs = outputs;
        render_pass.callback = Some(Box::new(DeclarativePassAdapter::new(pass)));

        self.add_pass(render_pass)
    }

    /// Create and add a new resource
    pub fn create_resource(
        &mut self,
        name: impl Into<String>,
        descriptor: ResourceDescriptor,
    ) -> ResourceId {
        let id = ResourceId(self.next_resource_id);
        self.next_resource_id += 1;

        let name_str = name.into();
        let resource = Resource::new(id, name_str.clone(), descriptor);

        self.resource_names.insert(name_str, id);
        self.resources.push(resource);

        id
    }

    /// Declare an image resource with the given descriptor
    ///
    /// This is a convenience method for creating image resources with a clean API.
    ///
    /// # Example
    /// ```ignore
    /// // Absolute size
    /// let color_id = graph.declare_image(
    ///     "color",
    ///     Format::Rgba8Unorm,
    ///     ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
    ///     ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
    ///     SampleCount::One,
    ///     1,
    /// );
    ///
    /// // Swapchain-relative
    /// let depth_id = graph.declare_image(
    ///     "depth",
    ///     Format::Depth32Float,
    ///     ExtentMode::Swapchain,
    ///     ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
    ///     SampleCount::One,
    ///     1,
    /// );
    ///
    /// // Half-size for performance
    /// let shadow_id = graph.declare_image(
    ///     "shadow",
    ///     Format::Depth32Float,
    ///     ExtentMode::SwapchainScaled(0.5),
    ///     ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
    ///     SampleCount::One,
    ///     1,
    /// );
    /// ```
    pub fn declare_image(
        &mut self,
        name: impl Into<String>,
        format: Format,
        extent: ExtentMode,
        usage: ImageUsageFlags,
        samples: SampleCount,
        mip_levels: u32,
    ) -> ResourceId {
        self.create_resource(
            name,
            ResourceDescriptor::Image {
                format,
                extent,
                usage,
                samples,
                mip_levels,
            },
        )
    }

    /// Declare a buffer resource with the given descriptor
    ///
    /// This is a convenience method for creating buffer resources with a clean API.
    ///
    /// # Example
    /// ```ignore
    /// let uniform_id = graph.declare_buffer("camera_uniform", 256,
    ///     BufferUsageFlags::new(BufferUsageFlags::UNIFORM));
    /// ```
    pub fn declare_buffer(
        &mut self,
        name: impl Into<String>,
        size: usize,
        usage: BufferUsageFlags,
    ) -> ResourceId {
        self.create_resource(name, ResourceDescriptor::Buffer { size, usage })
    }

    /// Declare a sampler resource
    ///
    /// This is a convenience method for creating sampler resources.
    ///
    /// # Example
    /// ```ignore
    /// // Linear filtering sampler
    /// let linear_sampler = graph.declare_sampler(
    ///     "linear_sampler",
    ///     SamplerDescriptor::default()
    /// );
    ///
    /// // Nearest filtering for pixel art
    /// let nearest_sampler = graph.declare_sampler(
    ///     "nearest_sampler",
    ///     SamplerDescriptor {
    ///         min_filter: FilterMode::Nearest,
    ///         mag_filter: FilterMode::Nearest,
    ///         ..Default::default()
    ///     }
    /// );
    /// ```
    pub fn declare_sampler(
        &mut self,
        name: impl Into<String>,
        descriptor: SamplerDescriptor,
    ) -> ResourceId {
        self.create_resource(name, ResourceDescriptor::Sampler(descriptor))
    }

    /// Import an external buffer into the render graph
    ///
    /// This creates a ResourceId for an existing backend buffer, allowing it to be
    /// used in declarative passes. The buffer must be kept alive externally.
    ///
    /// # Example
    /// ```ignore
    /// let camera_buffer = backend.create_buffer(&desc)?;
    /// let camera_id = graph.import_buffer(
    ///     "camera_uniform",
    ///     128,  // size in bytes
    ///     BufferUsageFlags::new(BufferUsageFlags::UNIFORM)
    /// );
    /// ```
    pub fn import_buffer(
        &mut self,
        name: impl Into<String>,
        size: usize,
        usage: BufferUsageFlags,
    ) -> ResourceId {
        let id = ResourceId(self.next_resource_id);
        self.next_resource_id += 1;

        let name_str = name.into();
        let descriptor = ResourceDescriptor::Buffer { size, usage };
        let mut resource = Resource::new(id, name_str.clone(), descriptor);

        // Mark as external so the render graph doesn't try to create/destroy it
        resource.mark_external();

        self.resource_names.insert(name_str, id);
        self.resources.push(resource);

        id
    }

    /// Import an external image/texture into the render graph
    ///
    /// This creates a ResourceId for an existing backend texture, allowing it to be
    /// used in declarative passes. The texture must be kept alive externally.
    ///
    /// # Example
    /// ```ignore
    /// let albedo_texture = backend.create_texture(&desc)?;
    /// let albedo_id = graph.import_image(
    ///     "albedo",
    ///     Format::Rgba8Unorm,
    ///     Extent3D::new_2d(1024, 1024),
    ///     ImageUsageFlags::new(ImageUsageFlags::SAMPLED),
    ///     SampleCount::One,
    ///     1
    /// );
    /// ```
    pub fn import_image(
        &mut self,
        name: impl Into<String>,
        format: Format,
        extent: crate::render_graph::resource::Extent3D,
        usage: ImageUsageFlags,
        samples: SampleCount,
        mip_levels: u32,
    ) -> ResourceId {
        let id = ResourceId(self.next_resource_id);
        self.next_resource_id += 1;

        let name_str = name.into();
        let descriptor = ResourceDescriptor::Image {
            format,
            extent: ExtentMode::Absolute(extent),
            usage,
            samples,
            mip_levels,
        };
        let mut resource = Resource::new(id, name_str.clone(), descriptor);

        // Mark as external so the render graph doesn't try to create/destroy it
        resource.mark_external();

        self.resource_names.insert(name_str, id);
        self.resources.push(resource);

        id
    }

    /// Get a pass by ID
    pub fn get_pass(&self, id: PassId) -> Option<&RenderPass> {
        self.passes.iter().find(|p| p.id == id)
    }

    /// Get a mutable pass by ID
    pub fn get_pass_mut(&mut self, id: PassId) -> Option<&mut RenderPass> {
        self.passes.iter_mut().find(|p| p.id == id)
    }

    /// Get a pass by name
    pub fn get_pass_by_name(&self, name: &str) -> Option<&RenderPass> {
        self.pass_names.get(name).and_then(|id| self.get_pass(*id))
    }

    /// Get a resource by ID
    pub fn get_resource(&self, id: ResourceId) -> Option<&Resource> {
        self.resources.iter().find(|r| r.id == id)
    }

    /// Get a mutable resource by ID
    pub fn get_resource_mut(&mut self, id: ResourceId) -> Option<&mut Resource> {
        self.resources.iter_mut().find(|r| r.id == id)
    }

    /// Get a resource by name
    pub fn get_resource_by_name(&self, name: &str) -> Option<&Resource> {
        self.resource_names
            .get(name)
            .and_then(|id| self.get_resource(*id))
    }

    /// Get next pass ID
    pub fn next_pass_id(&mut self) -> PassId {
        let id = PassId(self.next_pass_id);
        self.next_pass_id += 1;
        id
    }

    /// Get all passes
    pub fn passes(&self) -> &[RenderPass] {
        &self.passes
    }

    /// Get all resources
    pub fn resources(&self) -> &[Resource] {
        &self.resources
    }

    /// Register a shader with the graph
    ///
    /// Shaders registered here can be referenced by name in pipeline builders.
    ///
    /// # Example
    /// ```ignore
    /// graph.register_shader(
    ///     "forward.vert",
    ///     ShaderDescriptor::from_file("shaders/hlsl/forward.hlsl", ShaderStage::Vertex)
    ///         .with_entry_point("vs_main")
    /// );
    /// ```
    pub fn register_shader(
        &mut self,
        name: impl Into<String>,
        descriptor: ShaderDescriptor,
    ) -> ShaderHandle {
        self.shader_registry.register(name, descriptor)
    }

    /// Get the shader registry
    ///
    /// Allows direct access to the shader registry for advanced use cases.
    pub fn shader_registry(&self) -> &ShaderRegistry {
        &self.shader_registry
    }

    /// Get mutable shader registry
    ///
    /// Allows direct access to the shader registry for advanced use cases.
    pub fn shader_registry_mut(&mut self) -> &mut ShaderRegistry {
        &mut self.shader_registry
    }

    /// Get pipeline description for a pass
    ///
    /// Calls the pass's `declare_pipeline` method to build a pipeline description.
    /// This is used during graph compilation to create backend pipelines.
    ///
    /// # Arguments
    /// * `pass_id` - ID of the pass to get pipeline description for
    ///
    /// # Returns
    /// A PipelineBuilder containing the pipeline requirements, or None if the pass
    /// doesn't declare a pipeline (e.g., transfer passes)
    pub fn get_pipeline_description(&self, pass_id: PassId) -> Option<PipelineBuilder> {
        let pass = self.get_pass(pass_id)?;

        // Only graphics and compute passes have pipelines
        if pass.kind != PassKind::Graphics && pass.kind != PassKind::Compute {
            return None;
        }

        // Create a pipeline builder
        let mut builder = PipelineBuilder::new();

        // Let the pass declare its pipeline
        if let Some(callback) = &pass.callback {
            callback.declare_pipeline(&mut builder, &self.shader_registry);
        }

        Some(builder)
    }

    /// Build dependency edges between passes
    fn build_dependencies(&self) -> Vec<Dependency> {
        let mut dependencies = Vec::new();

        // Build producer map: resource -> pass that writes it
        let mut producers: HashMap<ResourceId, PassId> = HashMap::new();
        for pass in &self.passes {
            for output in &pass.outputs {
                producers.insert(output.resource, pass.id);
            }
        }

        // For each pass, find dependencies on other passes
        for pass in &self.passes {
            for input in &pass.inputs {
                if let Some(&producer) = producers.get(&input.resource) {
                    // This pass depends on the producer of this resource
                    if producer != pass.id {
                        dependencies.push(Dependency {
                            dependent: pass.id,
                            dependency: producer,
                            resource: input.resource,
                        });
                    }
                }
            }
        }

        dependencies
    }

    /// Perform topological sort using Kahn's algorithm
    fn topological_sort(&self, dependencies: &[Dependency]) -> Result<Vec<PassId>> {
        // Build adjacency list and in-degree map
        let mut in_degree: HashMap<PassId, usize> = HashMap::new();
        let mut adj_list: HashMap<PassId, Vec<PassId>> = HashMap::new();

        // Initialize all passes with in-degree 0
        for pass in &self.passes {
            in_degree.insert(pass.id, 0);
            adj_list.insert(pass.id, Vec::new());
        }

        // Build the graph
        for dep in dependencies {
            *in_degree.get_mut(&dep.dependent).unwrap() += 1;
            adj_list
                .get_mut(&dep.dependency)
                .unwrap()
                .push(dep.dependent);
        }

        // Queue of passes with no dependencies
        let mut queue: Vec<PassId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&id, _)| id)
            .collect();

        // Sort for deterministic results
        queue.sort();

        let mut result = Vec::new();

        while let Some(pass_id) = queue.pop() {
            result.push(pass_id);

            // Reduce in-degree of dependent passes
            if let Some(dependents) = adj_list.get(&pass_id) {
                for &dependent in dependents {
                    let degree = in_degree.get_mut(&dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push(dependent);
                        queue.sort();
                    }
                }
            }
        }

        // If we didn't process all passes, there's a cycle
        if result.len() != self.passes.len() {
            return Err(RenderGraphError::CyclicDependency);
        }

        Ok(result)
    }

    /// Compile the render graph
    ///
    /// This validates the graph and computes the execution order.
    pub fn compile(&mut self) -> Result<CompiledGraph> {
        // Validate that all resources have producers
        self.validate_producers()?;

        // Build dependency edges
        let dependencies = self.build_dependencies();

        // Compute execution order via topological sort
        let execution_order = self.topological_sort(&dependencies)?;

        // Update resource lifetimes based on execution order
        self.update_resource_lifetimes(&execution_order);

        // Build producer map
        let mut producers = HashMap::new();
        for pass in &self.passes {
            for output in &pass.outputs {
                producers.insert(output.resource, pass.id);
            }
        }

        // Insert barriers between passes
        let barriers = self.insert_barriers(&execution_order);

        // Collect pipeline descriptions from passes
        let mut pipeline_descriptions = HashMap::new();
        for &pass_id in &execution_order {
            if let Some(builder) = self.get_pipeline_description(pass_id) {
                log::debug!("Collected pipeline description for pass {:?}", pass_id);
                pipeline_descriptions.insert(pass_id, builder);
            }
        }

        // Collect resources that need allocation (non-external resources)
        let resources_to_allocate: Vec<ResourceId> = self
            .resources
            .iter()
            .filter(|r| !r.is_external())
            .map(|r| r.id)
            .collect();

        log::info!(
            "Graph compiled: {} passes, {} barriers, {} pipelines, {} resources to allocate",
            execution_order.len(),
            barriers.len(),
            pipeline_descriptions.len(),
            resources_to_allocate.len()
        );

        Ok(CompiledGraph {
            execution_order,
            producers,
            barriers,
            pipeline_descriptions,
            resources_to_allocate,
        })
    }

    /// Validate that all resources have producers
    fn validate_producers(&self) -> Result<()> {
        // Build producer count map
        let mut producer_count: HashMap<ResourceId, usize> = HashMap::new();

        for pass in &self.passes {
            for output in &pass.outputs {
                *producer_count.entry(output.resource).or_insert(0) += 1;
            }
        }

        // Check that all input resources have exactly one producer
        for pass in &self.passes {
            for input in &pass.inputs {
                let count = producer_count.get(&input.resource).copied().unwrap_or(0);

                if count == 0 {
                    let resource = self.get_resource(input.resource).ok_or_else(|| {
                        RenderGraphError::ResourceNotFound(format!("{}", input.resource))
                    })?;
                    return Err(RenderGraphError::NoProducer(resource.name.clone()));
                }

                if count > 1 {
                    let resource = self.get_resource(input.resource).ok_or_else(|| {
                        RenderGraphError::ResourceNotFound(format!("{}", input.resource))
                    })?;
                    return Err(RenderGraphError::MultipleProducers(resource.name.clone()));
                }
            }
        }

        Ok(())
    }

    /// Update resource lifetimes based on execution order
    fn update_resource_lifetimes(&mut self, execution_order: &[PassId]) {
        // Create a map from PassId to execution index
        let pass_to_index: HashMap<PassId, usize> = execution_order
            .iter()
            .enumerate()
            .map(|(idx, &pass_id)| (pass_id, idx))
            .collect();

        // Collect resource updates first to avoid borrow checker issues
        let mut updates: Vec<(ResourceId, usize)> = Vec::new();
        for pass in &self.passes {
            let pass_index = pass_to_index[&pass.id];
            for resource_id in pass.all_resources() {
                updates.push((resource_id, pass_index));
            }
        }

        // Apply updates
        for (resource_id, pass_index) in updates {
            if let Some(resource) = self.get_resource_mut(resource_id) {
                resource.lifetime.update(pass_index);
            }
        }
    }

    /// Insert barriers between passes based on resource access patterns
    fn insert_barriers(&self, execution_order: &[PassId]) -> Vec<Barrier> {
        let mut inserter = BarrierInserter::new();
        let mut barriers = Vec::new();

        // Build resource kind map
        let resource_kinds: HashMap<ResourceId, ResourceKind> =
            self.resources.iter().map(|r| (r.id, r.kind)).collect();

        // Analyze transitions between consecutive passes
        for window in execution_order.windows(2) {
            let src_pass_id = window[0];
            let dst_pass_id = window[1];

            if let (Some(src_pass), Some(dst_pass)) =
                (self.get_pass(src_pass_id), self.get_pass(dst_pass_id))
            {
                let barrier = inserter.analyze_transition(
                    src_pass_id,
                    dst_pass_id,
                    &src_pass.outputs,
                    &dst_pass.inputs,
                    &resource_kinds,
                );

                if !barrier.is_empty() {
                    barriers.push(barrier);
                }
            }
        }

        // Optimize barriers (merge, deduplicate, etc.)
        BarrierInserter::optimize_barriers(barriers)
    }
}

impl Default for RenderGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render_graph::pass::{
        AccessType, ImageLayout, PassKind, PipelineStage, ResourceAccess,
    };
    use crate::render_graph::resource::{Extent3D, Format, ImageUsageFlags, SampleCount};

    #[test]
    fn test_graph_creation() {
        let graph = RenderGraph::new();
        assert_eq!(graph.passes().len(), 0);
        assert_eq!(graph.resources().len(), 0);
    }

    #[test]
    fn test_resource_creation() {
        let mut graph = RenderGraph::new();

        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };

        let res_id = graph.create_resource("color_buffer", desc);
        assert_eq!(res_id, ResourceId(0));

        let resource = graph.get_resource(res_id).unwrap();
        assert_eq!(resource.name(), "color_buffer");
    }

    #[test]
    fn test_simple_graph_compilation() {
        let mut graph = RenderGraph::new();

        // Create a resource
        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color = graph.create_resource("color", desc);

        // Create a pass that writes to the resource
        let mut pass = RenderPass::new(graph.next_pass_id(), "render", PassKind::Graphics);
        pass.add_output(ResourceAccess::write(
            color,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));
        graph.add_pass(pass);

        // Should compile successfully
        let compiled = graph.compile().unwrap();
        assert_eq!(compiled.execution_order.len(), 1);
    }

    #[test]
    fn test_dependency_chain() {
        let mut graph = RenderGraph::new();

        // Create resources
        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let res1 = graph.create_resource("res1", desc.clone());
        let res2 = graph.create_resource("res2", desc);

        // Pass A produces res1
        let mut pass_a = RenderPass::new(graph.next_pass_id(), "pass_a", PassKind::Graphics);
        pass_a.add_output(ResourceAccess::write(
            res1,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));
        let pass_a_id = pass_a.id;
        graph.add_pass(pass_a);

        // Pass B reads res1, produces res2
        let mut pass_b = RenderPass::new(graph.next_pass_id(), "pass_b", PassKind::Graphics);
        pass_b.add_input(ResourceAccess::read(
            res1,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
        ));
        pass_b.add_output(ResourceAccess::write(
            res2,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));
        let pass_b_id = pass_b.id;
        graph.add_pass(pass_b);

        // Compile and check order
        let compiled = graph.compile().unwrap();
        assert_eq!(compiled.execution_order, vec![pass_a_id, pass_b_id]);
    }

    #[test]
    fn test_cyclic_dependency_detection() {
        let mut graph = RenderGraph::new();

        // Create resources
        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let res1 = graph.create_resource("res1", desc.clone());
        let res2 = graph.create_resource("res2", desc);

        // Pass A produces res1, reads res2
        let mut pass_a = RenderPass::new(graph.next_pass_id(), "pass_a", PassKind::Graphics);
        pass_a.add_output(ResourceAccess::write(
            res1,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));
        pass_a.add_input(ResourceAccess::read(
            res2,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
        ));
        graph.add_pass(pass_a);

        // Pass B produces res2, reads res1 - creates a cycle!
        let mut pass_b = RenderPass::new(graph.next_pass_id(), "pass_b", PassKind::Graphics);
        pass_b.add_output(ResourceAccess::write(
            res2,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));
        pass_b.add_input(ResourceAccess::read(
            res1,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
        ));
        graph.add_pass(pass_b);

        // Should detect the cycle
        let result = graph.compile();
        assert!(matches!(result, Err(RenderGraphError::CyclicDependency)));
    }

    #[test]
    fn test_resource_lifetime_tracking() {
        let mut graph = RenderGraph::new();

        // Create resources
        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let res1 = graph.create_resource("res1", desc.clone());
        let res2 = graph.create_resource("res2", desc);

        // Pass A produces res1
        let mut pass_a = RenderPass::new(graph.next_pass_id(), "pass_a", PassKind::Graphics);
        pass_a.add_output(ResourceAccess::write(
            res1,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));
        graph.add_pass(pass_a);

        // Pass B reads res1, produces res2
        let mut pass_b = RenderPass::new(graph.next_pass_id(), "pass_b", PassKind::Graphics);
        pass_b.add_input(ResourceAccess::read(
            res1,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
        ));
        pass_b.add_output(ResourceAccess::write(
            res2,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
        ));
        graph.add_pass(pass_b);

        // Pass C reads res1
        let mut pass_c = RenderPass::new(graph.next_pass_id(), "pass_c", PassKind::Graphics);
        pass_c.add_input(ResourceAccess::read(
            res1,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
        ));
        // Also read res2 to create a proper dependency chain
        pass_c.add_input(ResourceAccess::read(
            res2,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
        ));
        graph.add_pass(pass_c);

        // Compile to update lifetimes
        graph.compile().unwrap();

        // Check res1 lifetime (used in passes 0, 1, 2)
        let res1_resource = graph.get_resource(res1).unwrap();
        assert_eq!(res1_resource.lifetime.first_use, Some(0));
        assert_eq!(res1_resource.lifetime.last_use, Some(2));

        // Check res2 lifetime (used in passes 1, 2)
        let res2_resource = graph.get_resource(res2).unwrap();
        assert_eq!(res2_resource.lifetime.first_use, Some(1));
        assert_eq!(res2_resource.lifetime.last_use, Some(2));
    }

    #[test]
    fn test_barrier_insertion() {
        let mut graph = RenderGraph::new();

        // Create resources
        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let res1 = graph.create_resource("res1", desc.clone());
        let res2 = graph.create_resource("res2", desc);

        // Pass A produces res1
        let mut pass_a = RenderPass::new(graph.next_pass_id(), "pass_a", PassKind::Graphics);
        pass_a.add_output(ResourceAccess::new(
            res1,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));
        graph.add_pass(pass_a);

        // Pass B reads res1, produces res2
        let mut pass_b = RenderPass::new(graph.next_pass_id(), "pass_b", PassKind::Graphics);
        pass_b.add_input(ResourceAccess::new(
            res1,
            AccessType::Read,
            PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
            Some(ImageLayout::ShaderReadOnly),
        ));
        pass_b.add_output(ResourceAccess::new(
            res2,
            AccessType::Write,
            PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            Some(ImageLayout::ColorAttachment),
        ));
        graph.add_pass(pass_b);

        // Compile
        let compiled = graph.compile().unwrap();

        // Should have one barrier for the layout transition
        assert!(!compiled.barriers.is_empty());
        let barrier = &compiled.barriers[0];
        assert_eq!(barrier.src_pass, PassId(0));
        assert_eq!(barrier.dst_pass, PassId(1));
        assert!(!barrier.image_barriers.is_empty());
    }

    #[test]
    fn test_pipeline_description_collection() {
        use crate::render_graph::shader::{ShaderDescriptor, ShaderSource, ShaderStage};
        use crate::render_graph::DeclarativePass;

        let mut graph = RenderGraph::new();

        // Register test shaders
        graph.register_shader(
            "test.vert",
            ShaderDescriptor {
                source: ShaderSource::Embedded(&[0, 1, 2, 3]), // Dummy SPIR-V
                entry_point: "main",
                stage: ShaderStage::Vertex,
                backend_compile: false,
            },
        );
        graph.register_shader(
            "test.frag",
            ShaderDescriptor {
                source: ShaderSource::Embedded(&[4, 5, 6, 7]), // Dummy SPIR-V
                entry_point: "main",
                stage: ShaderStage::Fragment,
                backend_compile: false,
            },
        );

        // Create a resource
        let desc = ResourceDescriptor::Image {
            format: Format::Rgba8Unorm,
            extent: ExtentMode::Absolute(Extent3D::new_2d(1280, 720)),
            usage: ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
            samples: SampleCount::One,
            mip_levels: 1,
        };
        let color = graph.create_resource("color", desc);

        // Create a declarative test pass
        struct TestPass {
            output: ResourceId,
        }

        impl DeclarativePass for TestPass {
            fn name(&self) -> &str {
                "test_pass"
            }

            fn declare_dependencies(&self, builder: &mut crate::render_graph::PassBuilder) {
                use crate::render_graph::PipelineStage;
                builder.write(
                    self.output,
                    PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
                );
            }

            fn declare_pipeline(
                &self,
                builder: &mut crate::render_graph::PipelineBuilder,
                registry: &crate::render_graph::ShaderRegistry,
            ) {
                let vs_handle = registry.get_handle("test.vert").unwrap();
                let fs_handle = registry.get_handle("test.frag").unwrap();
                builder.vertex_shader(vs_handle).fragment_shader(fs_handle);
            }

            fn prepare(&self, _context: &mut dyn crate::render_graph::PassPreparationContext) {}

            fn execute(&self, _context: &mut dyn crate::render_graph::PassExecutionContext) {}
        }

        let test_pass = TestPass { output: color };
        let pass_id = graph.add_declarative_pass(test_pass);

        // Compile the graph
        let compiled = graph.compile().unwrap();

        // Verify pipeline description was collected
        assert_eq!(compiled.pipeline_descriptions.len(), 1);
        assert!(compiled.pipeline_descriptions.contains_key(&pass_id));

        // Verify the pipeline description contains the shaders
        let pipeline_desc = &compiled.pipeline_descriptions[&pass_id];
        let shaders = pipeline_desc.shaders();
        assert_eq!(shaders.len(), 2); // Vertex and fragment shader
    }
}

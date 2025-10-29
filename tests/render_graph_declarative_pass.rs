/// Tests for declarative pass API
///
/// Phase 2.1: DeclarativePass trait and PassBuilder
use rusty_renderer::render_graph::{
    DeclarativePass, Extent3D, ExtentMode, Format, ImageLayout, ImageUsageFlags, PassBuilder,
    PassExecutionContext, PassKind, PipelineStage, RenderGraph, SampleCount,
};

/// Example declarative pass for testing
struct TestPass {
    name: String,
    input: rusty_renderer::render_graph::ResourceId,
    output: rusty_renderer::render_graph::ResourceId,
}

impl DeclarativePass for TestPass {
    fn name(&self) -> &str {
        &self.name
    }

    fn kind(&self) -> PassKind {
        PassKind::Graphics
    }

    fn declare_dependencies(&self, builder: &mut PassBuilder) {
        builder
            .read(
                self.input,
                PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
            )
            .with_layout(ImageLayout::ShaderReadOnly)
            .write(
                self.output,
                PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
            )
            .with_layout(ImageLayout::ColorAttachment);
    }

    fn execute(&self, _ctx: &mut dyn PassExecutionContext) {
        // Test pass - no actual rendering
    }
}

#[test]
fn test_declarative_pass_basic() {
    let mut graph = RenderGraph::new();

    // Create resources
    let input = graph.declare_image(
        "input",
        Format::Rgba8Unorm,
        ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
        ImageUsageFlags::new(ImageUsageFlags::SAMPLED),
        SampleCount::One,
        1,
    );

    let output = graph.declare_image(
        "output",
        Format::Rgba8Unorm,
        ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
        ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
        SampleCount::One,
        1,
    );

    // Add declarative pass
    let pass = TestPass {
        name: "test_pass".to_string(),
        input,
        output,
    };

    let pass_id = graph.add_declarative_pass(pass);

    // Verify pass was added
    let render_pass = graph.get_pass(pass_id).expect("Pass should exist");
    assert_eq!(render_pass.name, "test_pass");
    assert_eq!(render_pass.kind, PassKind::Graphics);

    // Verify dependencies were configured
    assert_eq!(render_pass.inputs.len(), 1);
    assert_eq!(render_pass.outputs.len(), 1);

    assert_eq!(render_pass.inputs[0].resource, input);
    assert_eq!(render_pass.outputs[0].resource, output);

    // Verify layouts were set
    assert_eq!(
        render_pass.inputs[0].layout,
        Some(ImageLayout::ShaderReadOnly)
    );
    assert_eq!(
        render_pass.outputs[0].layout,
        Some(ImageLayout::ColorAttachment)
    );
}

#[test]
fn test_declarative_pass_with_resource_declaration() {
    /// Pass that creates its own resources
    struct SelfContainedPass {
        output_name: String,
    }

    impl SelfContainedPass {
        fn new() -> Self {
            Self {
                output_name: "internal_buffer".to_string(),
            }
        }
    }

    impl DeclarativePass for SelfContainedPass {
        fn name(&self) -> &str {
            "self_contained"
        }

        fn declare_resources(&self, graph: &mut RenderGraph) {
            // Declare resources during pass setup
            graph.declare_image(
                &self.output_name,
                Format::Rgba8Unorm,
                ExtentMode::Swapchain,
                ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
                SampleCount::One,
                1,
            );
        }

        fn declare_dependencies(&self, builder: &mut PassBuilder) {
            // In a real implementation, we'd look up the resource by name
            // For this test, we'll just not add dependencies since we can't
            // easily store the ID in a thread-safe way in the pass struct
            let _ = builder;
        }

        fn execute(&self, _ctx: &mut dyn PassExecutionContext) {
            // Test pass
        }
    }

    let mut graph = RenderGraph::new();
    let pass = SelfContainedPass::new();
    let _pass_id = graph.add_declarative_pass(pass);

    // Verify resource was created
    let resource = graph.get_resource_by_name("internal_buffer");
    assert!(resource.is_some());
}

#[test]
fn test_pass_builder_chaining() {
    let mut graph = RenderGraph::new();

    let res1 = graph.declare_image(
        "res1",
        Format::Rgba8Unorm,
        ExtentMode::Absolute(Extent3D::new_2d(256, 256)),
        ImageUsageFlags::new(ImageUsageFlags::SAMPLED),
        SampleCount::One,
        1,
    );

    let res2 = graph.declare_image(
        "res2",
        Format::Rgba8Unorm,
        ExtentMode::Absolute(Extent3D::new_2d(256, 256)),
        ImageUsageFlags::new(ImageUsageFlags::STORAGE),
        SampleCount::One,
        1,
    );

    let res3 = graph.declare_image(
        "res3",
        Format::Rgba8Unorm,
        ExtentMode::Absolute(Extent3D::new_2d(256, 256)),
        ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
        SampleCount::One,
        1,
    );

    struct MultiResourcePass {
        res1: rusty_renderer::render_graph::ResourceId,
        res2: rusty_renderer::render_graph::ResourceId,
        res3: rusty_renderer::render_graph::ResourceId,
    }

    impl DeclarativePass for MultiResourcePass {
        fn name(&self) -> &str {
            "multi_resource"
        }

        fn declare_dependencies(&self, builder: &mut PassBuilder) {
            builder
                .read(
                    self.res1,
                    PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
                )
                .with_layout(ImageLayout::ShaderReadOnly)
                .read_write(self.res2, PipelineStage::new(PipelineStage::COMPUTE_SHADER))
                .with_layout(ImageLayout::General)
                .write(
                    self.res3,
                    PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
                )
                .with_layout(ImageLayout::ColorAttachment);
        }

        fn execute(&self, _ctx: &mut dyn PassExecutionContext) {}
    }

    let pass = MultiResourcePass { res1, res2, res3 };

    let pass_id = graph.add_declarative_pass(pass);
    let render_pass = graph.get_pass(pass_id).unwrap();

    // Verify multiple inputs/outputs
    assert_eq!(render_pass.inputs.len(), 1); // res1
    assert_eq!(render_pass.outputs.len(), 2); // res2 and res3
}

#[test]
fn test_multiple_declarative_passes() {
    let mut graph = RenderGraph::new();

    let tex = graph.declare_image(
        "texture",
        Format::Rgba8Unorm,
        ExtentMode::Absolute(Extent3D::new_2d(512, 512)),
        ImageUsageFlags::new(ImageUsageFlags::SAMPLED | ImageUsageFlags::COLOR_ATTACHMENT),
        SampleCount::One,
        1,
    );

    struct Pass1 {
        output: rusty_renderer::render_graph::ResourceId,
    }
    impl DeclarativePass for Pass1 {
        fn name(&self) -> &str {
            "pass1"
        }
        fn declare_dependencies(&self, builder: &mut PassBuilder) {
            builder
                .write(
                    self.output,
                    PipelineStage::new(PipelineStage::COLOR_ATTACHMENT_OUTPUT),
                )
                .with_layout(ImageLayout::ColorAttachment);
        }
        fn execute(&self, _ctx: &mut dyn PassExecutionContext) {}
    }

    struct Pass2 {
        input: rusty_renderer::render_graph::ResourceId,
    }
    impl DeclarativePass for Pass2 {
        fn name(&self) -> &str {
            "pass2"
        }
        fn declare_dependencies(&self, builder: &mut PassBuilder) {
            builder
                .read(
                    self.input,
                    PipelineStage::new(PipelineStage::FRAGMENT_SHADER),
                )
                .with_layout(ImageLayout::ShaderReadOnly);
        }
        fn execute(&self, _ctx: &mut dyn PassExecutionContext) {}
    }

    let _pass1_id = graph.add_declarative_pass(Pass1 { output: tex });
    let _pass2_id = graph.add_declarative_pass(Pass2 { input: tex });

    // Verify both passes exist
    assert!(graph.get_pass_by_name("pass1").is_some());
    assert!(graph.get_pass_by_name("pass2").is_some());
}

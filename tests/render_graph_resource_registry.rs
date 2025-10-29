/// Tests for render graph resource registry
/// 
/// Phase 1.1: Resource registry and name-based lookup

use rusty_renderer::render_graph::{
    BufferUsageFlags, Extent3D, Format, ImageUsageFlags, RenderGraph, SampleCount,
};

#[test]
fn test_declare_image() {
    let mut graph = RenderGraph::new();

    // Declare an image using the convenience method
    let depth_id = graph.declare_image(
        "depth",
        Format::Depth32Float,
        Extent3D::new_2d(800, 600),
        ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        SampleCount::One,
    );

    // Verify the resource exists
    let resource = graph.get_resource(depth_id).expect("Resource should exist");
    assert_eq!(resource.name(), "depth");
    assert!(resource.is_image());

    // Verify we can look it up by name
    let resource_by_name = graph
        .get_resource_by_name("depth")
        .expect("Should find by name");
    assert_eq!(resource_by_name.id, depth_id);
}

#[test]
fn test_declare_buffer() {
    let mut graph = RenderGraph::new();

    // Declare a buffer using the convenience method
    let uniform_id = graph.declare_buffer(
        "camera_uniform",
        256,
        BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
    );

    // Verify the resource exists
    let resource = graph
        .get_resource(uniform_id)
        .expect("Resource should exist");
    assert_eq!(resource.name(), "camera_uniform");
    assert!(resource.is_buffer());

    // Verify we can look it up by name
    let resource_by_name = graph
        .get_resource_by_name("camera_uniform")
        .expect("Should find by name");
    assert_eq!(resource_by_name.id, uniform_id);
}

#[test]
fn test_declare_sampler() {
    let mut graph = RenderGraph::new();

    // Declare a sampler using the convenience method
    let sampler_id = graph.declare_sampler("linear_sampler");

    // Verify the resource exists
    let resource = graph
        .get_resource(sampler_id)
        .expect("Resource should exist");
    assert_eq!(resource.name(), "linear_sampler");

    // Verify we can look it up by name
    let resource_by_name = graph
        .get_resource_by_name("linear_sampler")
        .expect("Should find by name");
    assert_eq!(resource_by_name.id, sampler_id);
}

#[test]
fn test_multiple_resources() {
    let mut graph = RenderGraph::new();

    // Declare multiple resources
    let color_id = graph.declare_image(
        "color",
        Format::Rgba8Unorm,
        Extent3D::new_2d(1920, 1080),
        ImageUsageFlags::new(
            ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED,
        ),
        SampleCount::One,
    );

    let depth_id = graph.declare_image(
        "depth",
        Format::Depth32Float,
        Extent3D::new_2d(1920, 1080),
        ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        SampleCount::One,
    );

    let uniform_id = graph.declare_buffer(
        "uniforms",
        512,
        BufferUsageFlags::new(BufferUsageFlags::UNIFORM),
    );

    let vertex_id = graph.declare_buffer(
        "vertices",
        4096,
        BufferUsageFlags::new(BufferUsageFlags::VERTEX),
    );

    // Verify all resources exist and have unique IDs
    assert_ne!(color_id, depth_id);
    assert_ne!(color_id, uniform_id);
    assert_ne!(depth_id, uniform_id);
    assert_ne!(uniform_id, vertex_id);

    // Verify lookup by name works for all
    assert!(graph.get_resource_by_name("color").is_some());
    assert!(graph.get_resource_by_name("depth").is_some());
    assert!(graph.get_resource_by_name("uniforms").is_some());
    assert!(graph.get_resource_by_name("vertices").is_some());

    // Verify missing resource returns None
    assert!(graph.get_resource_by_name("nonexistent").is_none());
}

#[test]
fn test_resource_kinds() {
    let mut graph = RenderGraph::new();

    let image_id = graph.declare_image(
        "test_image",
        Format::Rgba8Unorm,
        Extent3D::new_2d(256, 256),
        ImageUsageFlags::new(ImageUsageFlags::SAMPLED),
        SampleCount::One,
    );

    let buffer_id = graph.declare_buffer(
        "test_buffer",
        1024,
        BufferUsageFlags::new(BufferUsageFlags::STORAGE),
    );

    let sampler_id = graph.declare_sampler("test_sampler");

    // Verify resource kinds
    let image = graph.get_resource(image_id).unwrap();
    assert!(image.is_image());
    assert!(!image.is_buffer());

    let buffer = graph.get_resource(buffer_id).unwrap();
    assert!(buffer.is_buffer());
    assert!(!buffer.is_image());

    let sampler = graph.get_resource(sampler_id).unwrap();
    assert!(!sampler.is_image());
    assert!(!sampler.is_buffer());
}

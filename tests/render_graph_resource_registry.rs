/// Tests for render graph resource registry
///
/// Phase 1.1: Resource registry and name-based lookup
/// Phase 1.2: Extended descriptors with ExtentMode
use rusty_renderer::render_graph::{
    BufferUsageFlags, Extent3D, ExtentMode, Format, ImageUsageFlags, RenderGraph, SampleCount,
    SamplerDescriptor,
};

#[test]
fn test_declare_image() {
    let mut graph = RenderGraph::new();

    // Declare an image using the convenience method
    let depth_id = graph.declare_image(
        "depth",
        Format::Depth32Float,
        ExtentMode::Absolute(Extent3D::new_2d(800, 600)),
        ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        SampleCount::One,
        1,
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
    let sampler_id = graph.declare_sampler("linear_sampler", SamplerDescriptor::default());

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
        ExtentMode::Absolute(Extent3D::new_2d(1920, 1080)),
        ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT | ImageUsageFlags::SAMPLED),
        SampleCount::One,
        1,
    );

    let depth_id = graph.declare_image(
        "depth",
        Format::Depth32Float,
        ExtentMode::Absolute(Extent3D::new_2d(1920, 1080)),
        ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        SampleCount::One,
        1,
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
        ExtentMode::Absolute(Extent3D::new_2d(256, 256)),
        ImageUsageFlags::new(ImageUsageFlags::SAMPLED),
        SampleCount::One,
        1,
    );

    let buffer_id = graph.declare_buffer(
        "test_buffer",
        1024,
        BufferUsageFlags::new(BufferUsageFlags::STORAGE),
    );

    let sampler_id = graph.declare_sampler("test_sampler", SamplerDescriptor::default());

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

#[test]
fn test_extent_mode_swapchain() {
    let mut graph = RenderGraph::new();

    // Swapchain-sized image
    let color_id = graph.declare_image(
        "color",
        Format::Rgba8Unorm,
        ExtentMode::Swapchain,
        ImageUsageFlags::new(ImageUsageFlags::COLOR_ATTACHMENT),
        SampleCount::One,
        1,
    );

    let resource = graph.get_resource(color_id).unwrap();
    assert_eq!(resource.name(), "color");
}

#[test]
fn test_extent_mode_scaled() {
    let mut graph = RenderGraph::new();

    // Half-size shadow map
    let shadow_id = graph.declare_image(
        "shadow",
        Format::Depth32Float,
        ExtentMode::SwapchainScaled(0.5),
        ImageUsageFlags::new(ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT),
        SampleCount::One,
        1,
    );

    let resource = graph.get_resource(shadow_id).unwrap();
    assert_eq!(resource.name(), "shadow");
}

#[test]
fn test_extent_mode_resolution() {
    use rusty_renderer::render_graph::ExtentMode;

    // Test Absolute
    let absolute = ExtentMode::Absolute(Extent3D::new_2d(1920, 1080));
    let resolved = absolute.resolve(800, 600);
    assert_eq!(resolved.width, 1920);
    assert_eq!(resolved.height, 1080);

    // Test Swapchain
    let swapchain = ExtentMode::Swapchain;
    let resolved = swapchain.resolve(1280, 720);
    assert_eq!(resolved.width, 1280);
    assert_eq!(resolved.height, 720);

    // Test SwapchainScaled
    let scaled = ExtentMode::SwapchainScaled(0.5);
    let resolved = scaled.resolve(1920, 1080);
    assert_eq!(resolved.width, 960);
    assert_eq!(resolved.height, 540);

    // Test SwapchainScaled with 2x
    let scaled_2x = ExtentMode::SwapchainScaled(2.0);
    let resolved = scaled_2x.resolve(800, 600);
    assert_eq!(resolved.width, 1600);
    assert_eq!(resolved.height, 1200);
}

#[test]
fn test_mip_levels() {
    let mut graph = RenderGraph::new();

    // Texture with mipmaps
    let texture_id = graph.declare_image(
        "texture",
        Format::Rgba8Unorm,
        ExtentMode::Absolute(Extent3D::new_2d(1024, 1024)),
        ImageUsageFlags::new(ImageUsageFlags::SAMPLED | ImageUsageFlags::TRANSFER_DST),
        SampleCount::One,
        10, // Full mip chain for 1024x1024
    );

    let resource = graph.get_resource(texture_id).unwrap();
    assert_eq!(resource.name(), "texture");
}

#[test]
fn test_sampler_descriptor() {
    use rusty_renderer::render_graph::{AddressMode, FilterMode, SamplerDescriptor};

    let mut graph = RenderGraph::new();

    // Custom sampler for nearest filtering
    let nearest_sampler = graph.declare_sampler(
        "nearest",
        SamplerDescriptor {
            min_filter: FilterMode::Nearest,
            mag_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            max_anisotropy: 1.0,
        },
    );

    let resource = graph.get_resource(nearest_sampler).unwrap();
    assert_eq!(resource.name(), "nearest");

    // Default sampler (linear filtering)
    let linear_sampler = graph.declare_sampler("linear", SamplerDescriptor::default());

    let resource = graph.get_resource(linear_sampler).unwrap();
    assert_eq!(resource.name(), "linear");
}

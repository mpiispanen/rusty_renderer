//! Integration tests for resource management

use rusty_renderer::backends::*;

#[test]
#[ignore] // Requires GPU
fn test_buffer_creation_vulkan() {
    let mut backend = create_backend(BackendType::Vulkan, false).unwrap();
    backend.initialize_headless(800, 600).unwrap();

    // Create vertex buffer
    let vertex_desc = BufferDescriptor {
        size: 1024,
        usage: BufferUsage::vertex(),
        memory_location: MemoryLocation::GpuOnly,
        label: Some("Test Vertex Buffer".to_string()),
    };

    let buffer = backend.create_buffer(&vertex_desc).unwrap();
    assert_eq!(buffer.size(), 1024);
    assert_eq!(buffer.memory_location(), MemoryLocation::GpuOnly);

    backend.cleanup();
}

#[test]
#[ignore] // Requires GPU
fn test_buffer_upload_vulkan() {
    let mut backend = create_backend(BackendType::Vulkan, false).unwrap();
    backend.initialize_headless(800, 600).unwrap();

    // Create buffer
    let desc = BufferDescriptor {
        size: 256,
        usage: BufferUsage::vertex(),
        memory_location: MemoryLocation::GpuOnly,
        label: Some("Test Upload Buffer".to_string()),
    };

    let buffer = backend.create_buffer(&desc).unwrap();

    // Upload data
    let data: Vec<u8> = (0..256).map(|i| i as u8).collect();
    backend.upload_to_buffer(buffer.as_ref(), &data, 0).unwrap();

    backend.cleanup();
}

#[test]
#[ignore] // Requires GPU
fn test_texture_creation_vulkan() {
    let mut backend = create_backend(BackendType::Vulkan, false).unwrap();
    backend.initialize_headless(800, 600).unwrap();

    // Create texture
    let desc = TextureDescriptor {
        width: 256,
        height: 256,
        format: TextureFormat::Rgba8Srgb,
        usage: TextureUsage::sampled(),
        mip_levels: 1,
        label: Some("Test Texture".to_string()),
    };

    let texture = backend.create_texture(&desc).unwrap();
    assert_eq!(texture.width(), 256);
    assert_eq!(texture.height(), 256);
    assert_eq!(texture.format(), TextureFormat::Rgba8Srgb);

    backend.cleanup();
}

#[test]
#[ignore] // Requires GPU
fn test_texture_upload_vulkan() {
    let mut backend = create_backend(BackendType::Vulkan, false).unwrap();
    backend.initialize_headless(800, 600).unwrap();

    // Create texture
    let desc = TextureDescriptor {
        width: 64,
        height: 64,
        format: TextureFormat::Rgba8Srgb,
        usage: TextureUsage::sampled(),
        mip_levels: 1,
        label: Some("Test Texture Upload".to_string()),
    };

    let texture = backend.create_texture(&desc).unwrap();

    // Create test data (red checkerboard pattern)
    let size = (64 * 64 * 4) as usize;
    let mut data = vec![0u8; size];
    for y in 0..64 {
        for x in 0..64 {
            let idx = (y * 64 + x) * 4;
            let is_red = ((x / 8) + (y / 8)) % 2 == 0;
            data[idx] = if is_red { 255 } else { 0 };     // R
            data[idx + 1] = 0;                             // G
            data[idx + 2] = 0;                             // B
            data[idx + 3] = 255;                           // A
        }
    }

    // Upload texture data
    backend.upload_to_texture(texture.as_ref(), &data, 0).unwrap();

    backend.cleanup();
}

#[test]
#[ignore] // Requires GPU
fn test_sampler_creation_vulkan() {
    let mut backend = create_backend(BackendType::Vulkan, false).unwrap();
    backend.initialize_headless(800, 600).unwrap();

    // Create sampler
    let desc = SamplerDescriptor {
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: FilterMode::Linear,
        address_mode_u: AddressMode::Repeat,
        address_mode_v: AddressMode::Repeat,
        address_mode_w: AddressMode::Repeat,
        label: Some("Test Sampler".to_string()),
    };

    let _sampler = backend.create_sampler(&desc).unwrap();

    backend.cleanup();
}

//! Simple texture test
//!
//! This example verifies that the texture loading pipeline works correctly.
//! It loads a texture, creates it on the GPU, and validates the process.

use anyhow::{Context, Result};
use rusty_renderer::backends::{
    create_backend, BackendType, SamplerDescriptor, TextureDescriptor, TextureFormat, TextureUsage,
};
use rusty_renderer::resources::TextureLoader;
use std::path::Path;

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("=== Simple Texture Test ===");

    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let backend_name = args.get(1).map(|s| s.as_str()).unwrap_or("vulkan");

    let backend_type = match backend_name {
        "vulkan" => BackendType::Vulkan,
        "directx" | "dx12" => BackendType::DirectX12,
        "wgpu" => BackendType::Wgpu,
        _ => {
            eprintln!("Unknown backend: {backend_name}");
            eprintln!("Usage: simple_texture_test [vulkan|dx12|wgpu]");
            std::process::exit(1);
        }
    };

    log::info!("Using backend: {backend_type}");

    // Create backend
    let mut backend = create_backend(backend_type, false).context("Failed to create backend")?;

    // Initialize in headless mode
    backend
        .initialize_headless(800, 600)
        .context("Failed to initialize backend")?;

    log::info!("✓ Backend initialized");

    // Load texture
    let texture_path = Path::new("assets/textures/test_checkerboard.png");
    log::info!("Loading texture: {}", texture_path.display());

    let loaded_image =
        TextureLoader::load_from_file(texture_path).context("Failed to load texture")?;

    log::info!(
        "✓ Loaded texture: {}x{} ({} bytes)",
        loaded_image.width,
        loaded_image.height,
        loaded_image.data.len()
    );

    // Create texture on GPU
    let texture_desc = TextureDescriptor {
        width: loaded_image.width,
        height: loaded_image.height,
        format: TextureFormat::Rgba8Srgb,
        usage: TextureUsage::sampled(),
        mip_levels: 1,
        initial_data: Some(&loaded_image.data),
        label: Some("Test Checkerboard".to_string()),
    };

    let texture = backend
        .create_texture(&texture_desc)
        .context("Failed to create texture")?;

    log::info!("✓ Texture created on GPU");
    log::info!("  Format: {:?}", texture.format());
    log::info!("  Size: {}x{}", texture.width(), texture.height());
    log::info!("  Mip levels: {}", texture.mip_levels());

    // Create sampler
    let sampler_desc = SamplerDescriptor::default();
    let sampler = backend
        .create_sampler(&sampler_desc)
        .context("Failed to create sampler")?;

    log::info!("✓ Sampler created");

    // Cleanup
    drop(texture);
    drop(sampler);
    backend.cleanup();

    log::info!("✓ Resources cleaned up");
    log::info!("");
    log::info!("=== Test Complete ===");
    log::info!("  Backend: {backend_type}");
    log::info!("  Texture: {}x{}", loaded_image.width, loaded_image.height);
    log::info!("  All operations successful!");

    Ok(())
}

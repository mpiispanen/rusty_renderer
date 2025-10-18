//! Visual regression tests comparing backend outputs
//!
//! These tests render the triangle with different backends and compare
//! the outputs to ensure visual consistency.

use rusty_renderer::testing::ImageComparator;
use rusty_renderer::{backends, config::Backend, RenderConfig};
use std::path::PathBuf;

/// Test output directory
fn test_output_dir() -> PathBuf {
    let dir = PathBuf::from("target/visual_tests");
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Render triangle with a specific backend and save screenshot
fn render_with_backend(backend: Backend, output_path: &str) -> anyhow::Result<()> {
    let config = RenderConfig {
        backend,
        scene: "triangle".to_string(),
        width: 800,
        height: 600,
        debug: false,
        vsync: false,
        log_level: log::LevelFilter::Warn,
        max_frames: Some(1),
        headless: true,
        screenshot: Some(PathBuf::from(output_path)),
        screenshot_interval: 0,
    };

    // Initialize backend
    let mut backend = backends::create_backend(
        match config.backend {
            Backend::Vulkan => backends::BackendType::Vulkan,
            #[cfg(target_os = "windows")]
            Backend::DirectX => backends::BackendType::DirectX12,
            Backend::Wgpu => backends::BackendType::Wgpu,
        },
        config.debug,
    )?;

    // Initialize in headless mode
    backend.initialize_headless(config.width, config.height)?;

    // Render one frame
    backend.begin_frame()?;
    backend.end_frame()?;

    // Capture screenshot
    let (width, height, pixels) = backend.capture_frame()?;

    // Save as PNG
    use image::ImageBuffer;
    let img = ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, pixels)
        .ok_or_else(|| anyhow::anyhow!("Failed to create image from pixels"))?;

    // Create parent directory if needed
    if let Some(parent) = PathBuf::from(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }

    img.save(output_path)?;

    // Cleanup
    backend.cleanup();

    Ok(())
}

#[test]
#[ignore] // Run with: cargo test --test visual_tests -- --ignored
fn test_vulkan_vs_wgpu() {
    let _ = env_logger::builder().is_test(true).try_init();

    let output_dir = test_output_dir();
    let vulkan_path = output_dir.join("vulkan_triangle.png");
    let wgpu_path = output_dir.join("wgpu_triangle.png");
    let diff_path = output_dir.join("vulkan_vs_wgpu_diff.png");

    // Render with Vulkan
    println!("Rendering with Vulkan...");
    render_with_backend(Backend::Vulkan, vulkan_path.to_str().unwrap())
        .expect("Vulkan rendering failed");

    // Render with wgpu
    println!("Rendering with wgpu...");
    render_with_backend(Backend::Wgpu, wgpu_path.to_str().unwrap())
        .expect("wgpu rendering failed");

    // Compare images
    println!("Comparing images...");
    // Note: Different backends may have slight rendering differences due to:
    // - Coordinate system conventions (Y-up vs Y-down)
    // - Rasterization rules
    // - Precision differences
    // Using 15% tolerance and 10 units per channel to account for these
    let comparator = ImageComparator::new(15.0, 10);
    let result = comparator
        .compare_files(&vulkan_path, &wgpu_path)
        .expect("Image comparison failed");

    println!("Comparison result:");
    println!("  Matching pixels: {}/{}", 
        result.total_pixels - result.diff_pixels, 
        result.total_pixels);
    println!("  Different pixels: {} ({:.2}%)", 
        result.diff_pixels, 
        result.diff_percentage);
    println!("  MSE: {:.2}", result.mse);
    println!("  PSNR: {:.2} dB", result.psnr);

    // Generate diff image if there are differences
    if !result.matches {
        println!("Generating diff image...");
        let vulkan_img = image::open(&vulkan_path).unwrap().to_rgba8();
        let wgpu_img = image::open(&wgpu_path).unwrap().to_rgba8();
        
        comparator
            .generate_diff_file(&vulkan_img, &wgpu_img, &diff_path)
            .expect("Failed to generate diff");
        
        println!("Diff image saved to: {}", diff_path.display());
    }

    // Assert within tolerance
    assert!(
        comparator.is_within_tolerance(&result),
        "Images differ by {:.2}% (threshold: {:.2}%)",
        result.diff_percentage,
        comparator.tolerance_percentage
    );
}

#[test]
#[ignore]
#[cfg(target_os = "windows")]
fn test_vulkan_vs_directx() {
    let _ = env_logger::builder().is_test(true).try_init();

    let output_dir = test_output_dir();
    let vulkan_path = output_dir.join("vulkan_triangle.png");
    let dx_path = output_dir.join("directx_triangle.png");
    let diff_path = output_dir.join("vulkan_vs_directx_diff.png");

    // Render with Vulkan
    println!("Rendering with Vulkan...");
    render_with_backend(Backend::Vulkan, vulkan_path.to_str().unwrap())
        .expect("Vulkan rendering failed");

    // Render with DirectX
    println!("Rendering with DirectX...");
    render_with_backend(Backend::DirectX, dx_path.to_str().unwrap())
        .expect("DirectX rendering failed");

    // Compare images
    println!("Comparing images...");
    let comparator = ImageComparator::new(1.0, 5);
    let result = comparator
        .compare_files(&vulkan_path, &dx_path)
        .expect("Image comparison failed");

    println!("Comparison result:");
    println!("  Different pixels: {} ({:.2}%)", 
        result.diff_pixels, 
        result.diff_percentage);
    println!("  MSE: {:.2}", result.mse);
    println!("  PSNR: {:.2} dB", result.psnr);

    // Generate diff if needed
    if !result.matches {
        let vulkan_img = image::open(&vulkan_path).unwrap().to_rgba8();
        let dx_img = image::open(&dx_path).unwrap().to_rgba8();
        
        comparator
            .generate_diff_file(&vulkan_img, &dx_img, &diff_path)
            .expect("Failed to generate diff");
        
        println!("Diff image saved to: {}", diff_path.display());
    }

    // Assert within tolerance
    assert!(
        comparator.is_within_tolerance(&result),
        "Images differ by {:.2}% (threshold: {:.2}%)",
        result.diff_percentage,
        comparator.tolerance_percentage
    );
}

#[test]
#[ignore]
fn test_backend_consistency_all() {
    let _ = env_logger::builder().is_test(true).try_init();

    let output_dir = test_output_dir();
    
    // Render with all available backends
    let backends_to_test = vec![
        (Backend::Vulkan, "vulkan_triangle.png"),
        (Backend::Wgpu, "wgpu_triangle.png"),
        #[cfg(target_os = "windows")]
        (Backend::DirectX, "directx_triangle.png"),
    ];

    let mut paths = Vec::new();
    
    for (backend, filename) in &backends_to_test {
        let path = output_dir.join(filename);
        println!("Rendering with {:?}...", backend);
        render_with_backend(*backend, path.to_str().unwrap())
            .expect(&format!("{:?} rendering failed", backend));
        paths.push(path);
    }

    // Compare all pairs
    let comparator = ImageComparator::new(1.0, 5);
    
    for i in 0..paths.len() {
        for j in (i + 1)..paths.len() {
            println!(
                "\nComparing {} vs {}...",
                paths[i].file_name().unwrap().to_string_lossy(),
                paths[j].file_name().unwrap().to_string_lossy()
            );
            
            let result = comparator
                .compare_files(&paths[i], &paths[j])
                .expect("Comparison failed");
            
            println!("  Difference: {:.2}%", result.diff_percentage);
            println!("  PSNR: {:.2} dB", result.psnr);
            
            assert!(
                comparator.is_within_tolerance(&result),
                "Backends produce inconsistent output: {:.2}% difference",
                result.diff_percentage
            );
        }
    }
    
    println!("\n✓ All backends produce consistent output!");
}

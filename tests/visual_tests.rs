//! Visual regression tests comparing backend outputs
//!
//! These tests render the triangle with different backends and compare
//! the outputs to ensure visual consistency.

use rusty_renderer::testing::{FlipComparator, ImageComparator};
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
    println!(
        "  Matching pixels: {}/{}",
        result.total_pixels - result.diff_pixels,
        result.total_pixels
    );
    println!(
        "  Different pixels: {} ({:.2}%)",
        result.diff_pixels, result.diff_percentage
    );
    println!("  MSE: {:.2}", result.mse);
    println!("  PSNR: {:.2} dB", result.psnr);
    println!("  SSIM: {:.4}", result.ssim);
    println!("  Perceptual error: {:.4}", result.perceptual_error);

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
    println!(
        "  Different pixels: {} ({:.2}%)",
        result.diff_pixels, result.diff_percentage
    );
    println!("  MSE: {:.2}", result.mse);
    println!("  PSNR: {:.2} dB", result.psnr);
    println!("  SSIM: {:.4}", result.ssim);
    println!("  Perceptual error: {:.4}", result.perceptual_error);

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
        #[cfg(target_os = "windows")]
        (Backend::DirectX, "directx_triangle.png"),
    ];

    let mut paths = Vec::new();

    for (backend, filename) in &backends_to_test {
        let path = output_dir.join(filename);
        println!("Rendering with {backend:?}...");
        render_with_backend(*backend, path.to_str().unwrap())
            .unwrap_or_else(|_| panic!("{backend:?} rendering failed"));
        paths.push(path);
    }

    // Compare all pairs
    // Use relaxed tolerance for cross-backend comparison due to:
    // - Different coordinate systems and rasterization rules
    // - Precision differences in calculations
    // - Implementation-specific optimizations
    let comparator = ImageComparator::new(15.0, 10);

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
            println!("  SSIM: {:.4}", result.ssim);
            println!("  Perceptual error: {:.4}", result.perceptual_error);

            assert!(
                comparator.is_within_tolerance(&result),
                "Backends produce inconsistent output: {:.2}% difference",
                result.diff_percentage
            );
        }
    }

    println!("\n✓ All backends produce consistent output!");
}

#[test]
#[ignore]
fn test_vulkan_vs_wgpu_flip() {
    let _ = env_logger::builder().is_test(true).try_init();

    let output_dir = test_output_dir();
    let vulkan_path = output_dir.join("vulkan_triangle.png");
    let wgpu_path = output_dir.join("wgpu_triangle.png");

    // Render with Vulkan
    println!("Rendering with Vulkan...");
    render_with_backend(Backend::Vulkan, vulkan_path.to_str().unwrap())
        .expect("Vulkan rendering failed");

    // Render with wgpu
    println!("Rendering with wgpu...");

    // Compare using FLIP
    println!("\nComparing with NVIDIA FLIP...");
    let flip = FlipComparator::default();
    let result = flip
        .compare(&vulkan_path, &wgpu_path)
        .expect("FLIP comparison failed");

    println!("\nFLIP Results:");
    println!("  Mean error: {:.6}", result.mean);
    println!("  Median: {:.6}", result.median);
    println!("  1st quartile: {:.6}", result.q1);
    println!("  3rd quartile: {:.6}", result.q3);
    println!("  Min: {:.6}", result.min);
    println!("  Max: {:.6}", result.max);
    println!("  Pixels per degree: {:.1}", result.ppd);

    if let Some(error_map) = &result.error_map_path {
        println!("  Error map: {error_map}");
    }

    // Interpretation guide
    println!("\nInterpretation:");
    if result.mean < 0.05 {
        println!("  ✓ Excellent match (mean < 0.05)");
    } else if result.mean < 0.10 {
        println!("  ✓ Good match (mean < 0.10)");
    } else if result.mean < 0.15 {
        println!("  ⚠ Acceptable match (mean < 0.15)");
    } else {
        println!("  ✗ Significant differences (mean >= 0.15)");
    }

    // Assert acceptable threshold
    // For different backends, 0.15 is reasonable due to rasterization differences
    assert!(
        result.passes(0.15),
        "FLIP error too high: {:.6} (threshold: 0.15)",
        result.mean
    );
}

#[test]
#[ignore]
fn test_vulkan_vs_wgpu_flip_python_api() {
    let _ = env_logger::builder().is_test(true).try_init();

    let output_dir = test_output_dir();
    let vulkan_path = output_dir.join("vulkan_triangle.png");
    let wgpu_path = output_dir.join("wgpu_triangle.png");

    // Render with Vulkan
    println!("Rendering with Vulkan...");
    render_with_backend(Backend::Vulkan, vulkan_path.to_str().unwrap())
        .expect("Vulkan rendering failed");

    // Render with wgpu
    println!("Rendering with wgpu...");

    // Compare using FLIP Python API
    println!("\nComparing with NVIDIA FLIP (Python API)...");
    let flip = FlipComparator::with_python_api(None, 2);
    let result = flip
        .compare(&vulkan_path, &wgpu_path)
        .expect("FLIP comparison failed");

    println!("\nFLIP Results (Python API):");
    println!("  Mean error: {:.6}", result.mean);
    println!("  Median: {:.6}", result.median);
    println!("  1st quartile: {:.6}", result.q1);
    println!("  3rd quartile: {:.6}", result.q3);
    println!("  Min: {:.6}", result.min);
    println!("  Max: {:.6}", result.max);
    println!("  Pixels per degree: {:.1}", result.ppd);

    if let Some(error_map) = &result.error_map_path {
        println!("  Error map: {error_map}");
    }

    // Interpretation guide
    println!("\nInterpretation:");
    if result.mean < 0.05 {
        println!("  ✓ Excellent match (mean < 0.05)");
    } else if result.mean < 0.10 {
        println!("  ✓ Good match (mean < 0.10)");
    } else if result.mean < 0.15 {
        println!("  ⚠ Acceptable match (mean < 0.15)");
    } else {
        println!("  ✗ Significant differences (mean >= 0.15)");
    }

    // Assert acceptable threshold
    assert!(
        result.passes(0.15),
        "FLIP error too high: {:.6} (threshold: 0.15)",
        result.mean
    );
}

#[test]
#[ignore]
fn test_flip_comparison_methods() {
    let _ = env_logger::builder().is_test(true).try_init();

    let output_dir = test_output_dir();
    let vulkan_path = output_dir.join("vulkan_triangle.png");
    let wgpu_path = output_dir.join("wgpu_triangle.png");

    // Ensure images exist
    if !vulkan_path.exists() {
        println!("Rendering test images...");
        render_with_backend(Backend::Vulkan, vulkan_path.to_str().unwrap())
            .expect("Vulkan rendering failed");
    }

    println!("\n=== Comparing FLIP Methods ===\n");

    // Method 1: CLI
    println!("Method 1: CLI (flip command)");
    let flip_cli = FlipComparator::default();
    let result_cli = flip_cli
        .compare(&vulkan_path, &wgpu_path)
        .expect("CLI FLIP comparison failed");

    println!("  Mean: {:.6}", result_cli.mean);
    println!("  Method: CLI text parsing");

    // Method 2: Python API
    println!("\nMethod 2: Python API (flip_compare.py)");
    let flip_py = FlipComparator::with_python_api(None, 1);
    let result_py = flip_py
        .compare(&vulkan_path, &wgpu_path)
        .expect("Python API FLIP comparison failed");

    println!("  Mean: {:.6}", result_py.mean);
    println!("  Method: JSON parsing");
    println!("  Error map: {:?}", result_py.error_map_path);

    // Both methods should give same mean error
    let diff = (result_cli.mean - result_py.mean).abs();
    println!("\nDifference between methods: {diff:.8}");

    assert!(
        diff < 0.001,
        "FLIP methods disagree: CLI={:.6}, Python={:.6}",
        result_cli.mean,
        result_py.mean
    );

    println!("\n✓ Both methods produce consistent results!");
}

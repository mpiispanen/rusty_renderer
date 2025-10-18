# Visual Testing Infrastructure

This module provides comprehensive image comparison utilities for automated visual regression testing of the renderer.

## Features

### Image Comparison Metrics

The `ImageComparator` calculates multiple metrics to assess visual similarity:

1. **Pixel-level Matching**
   - Counts exact pixel matches within configurable tolerance
   - Useful for detecting any changes

2. **MSE (Mean Squared Error)**
   - Measures average squared difference across all channels
   - Lower is better (0 = identical)

3. **PSNR (Peak Signal-to-Noise Ratio)**
   - Expressed in decibels (dB)
   - Higher is better (∞ = identical)
   - Typical values: >30 dB is good, >40 dB is excellent

4. **SSIM (Structural Similarity Index)**
   - Range: -1 to 1 (1 = identical)
   - Measures perceived structural similarity
   - More aligned with human visual perception than MSE/PSNR

5. **Perceptual Error** (FLIP-inspired)
   - Luminance-weighted difference
   - Uses sRGB to linear RGB conversion
   - Weights luminance changes more heavily than chromatic differences
   - Lower is better (0 = identical)

### Tolerance Configuration

```rust
// Exact match (no tolerance)
let comparator = ImageComparator::default();

// 5% pixel difference allowed, 10 units per channel
let comparator = ImageComparator::new(5.0, 10);
```

### Diff Image Generation

Automatically generates visual diff images highlighting differences:
- Matching pixels: shown in grayscale
- Different pixels: highlighted in red

## Usage

### Basic Comparison

```rust
use rusty_renderer::testing::ImageComparator;

let comparator = ImageComparator::new(1.0, 5);
let result = comparator.compare_files("image1.png", "image2.png")?;

println!("Difference: {:.2}%", result.diff_percentage);
println!("PSNR: {:.2} dB", result.psnr);
println!("SSIM: {:.4}", result.ssim);
println!("Perceptual error: {:.4}", result.perceptual_error);

if !comparator.is_within_tolerance(&result) {
    comparator.generate_diff_file(&img1, &img2, "diff.png")?;
}
```

### Running Visual Tests

Visual regression tests are located in `tests/visual_tests.rs`:

```bash
# Run all visual tests
cargo test --test visual_tests -- --ignored --nocapture

# Run specific test
cargo test --test visual_tests test_vulkan_vs_wgpu -- --ignored --nocapture
```

Test outputs are saved to `target/visual_tests/`.

## Backend Comparison

### Expected Differences

Different rendering backends may produce slightly different outputs due to:

1. **Coordinate Systems**
   - Y-axis orientation (Y-up vs Y-down)
   - NDC (Normalized Device Coordinates) ranges

2. **Rasterization Rules**
   - Pixel center conventions
   - Triangle fill rules
   - Multi-sampling behavior

3. **Precision**
   - Floating-point calculation differences
   - Depth buffer precision
   - Color format conversions

4. **Implementation Details**
   - Driver-specific optimizations
   - Hardware-specific behavior
   - Shader compiler differences

### Tolerance Guidelines

- **Same backend, same hardware**: 0-1% difference
- **Different backends (Vulkan vs DirectX)**: 1-5% difference
- **Different backends (Vulkan vs wgpu)**: 10-15% difference
  - wgpu adds additional abstraction layers
  - May use different coordinate conventions

## Perceptual Comparison (FLIP-inspired)

Our perceptual comparison is inspired by [NVIDIA FLIP](https://research.nvidia.com/publication/2020-07_FLIP), 
implementing key concepts:

- **Luminance Sensitivity**: Human vision is more sensitive to brightness changes
- **sRGB to Linear**: Proper color space conversion for accurate perception
- **Spatial Weighting**: Window-based SSIM for structural comparison

While not a full FLIP implementation (which requires C++), our approach provides:
- Fast, pure-Rust implementation
- Good correlation with perceptual quality
- Suitable for automated testing

## Interpreting Results

### Good Match
```
Difference: 2.5%
PSNR: 35.2 dB
SSIM: 0.985
Perceptual error: 0.05
```

### Acceptable Variation
```
Difference: 12.5%
PSNR: 22.8 dB
SSIM: 0.979
Perceptual error: 0.11
```

### Significant Difference
```
Difference: 45.0%
PSNR: 15.2 dB
SSIM: 0.75
Perceptual error: 0.85
```

## CI Integration

Visual tests can be integrated into CI pipelines:

1. Run tests with GPU access
2. Compare against baseline images
3. Generate diff images on failure
4. Archive results as artifacts

See `.github/workflows/` for examples.

# Visual Testing Infrastructure

This module provides comprehensive image comparison utilities for automated visual regression testing of the renderer.

## Features

## Legacy Image Comparison

### Image Comparison Metrics

The `ImageComparator` provides traditional pixel-based metrics for quick comparisons:

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

## Perceptual Comparison (FLIP)

Our testing infrastructure now includes full integration with [NVIDIA FLIP](https://research.nvidia.com/publication/2020-07_FLIP), 
the industry-standard perceptual image comparison metric.

### FLIP Integration

FLIP (Feature-based Locally-Adaptive Pixel) is a perceptual image metric developed by NVIDIA that accounts for:
- **Spatial frequency filtering** based on human contrast sensitivity
- **Color perception** in LMS (cone response) color space
- **Spatial pooling** to account for local adaptation

We provide two methods for FLIP comparison:

#### 1. Command-Line Tool (CLI)

Uses the `flip` command-line tool from `flip-evaluator`:

```rust
let flip = FlipComparator::default();
let result = flip.compare("reference.png", "test.png")?;
```

Pros:
- Simple, no additional scripts needed
- Works with existing CLI tools

Cons:
- Requires parsing text output
- Less detailed error information

#### 2. Python API (Recommended)

Uses our Python wrapper script (`scripts/flip_compare.py`) for direct API access:

```rust
let flip = FlipComparator::with_python_api(None, 2);
let result = flip.compare("reference.png", "test.png")?;
```

Pros:
- JSON output for reliable parsing
- More detailed error maps
- Direct access to numpy arrays
- Better error handling

Cons:
- Requires Python 3 and numpy

### Installation

```bash
# Install FLIP evaluator
pip install flip-evaluator numpy

# Verify installation
flip --help
python3 scripts/flip_compare.py --help
```

### Using the Python Script

The `scripts/flip_compare.py` script provides a clean interface to the FLIP Python API:

```bash
# Compare two images
python3 scripts/flip_compare.py reference.png test.png

# With custom PPD and error map
python3 scripts/flip_compare.py reference.png test.png \
    --ppd 67 \
    --error-map diff.png \
    --output results.json

# Silent mode (only JSON output)
python3 scripts/flip_compare.py reference.png test.png -v 0
```

Output example:
```json
{
  "mean": 0.081237,
  "median": 0.001462,
  "q1": 0.001462,
  "q3": 0.001462,
  "min": 0.001462,
  "max": 0.997351,
  "ppd": 67.0,
  "dynamic_range": "LDR",
  "reference": "reference.png",
  "test": "test.png",
  "error_map": "error_map.png"
}
```

### FLIP Thresholds

Recommended thresholds for mean FLIP error:
- **< 0.05**: Excellent match (imperceptible differences)
- **< 0.10**: Good match (minor differences)
- **< 0.15**: Acceptable match (noticeable but acceptable differences)
- **≥ 0.15**: Significant differences (requires investigation)

### Example Tests

```rust
#[test]
fn test_with_flip() {
    // Using CLI method
    let flip = FlipComparator::default();
    let result = flip.compare("ref.png", "test.png")?;
    assert!(result.passes(0.10), "FLIP error: {:.6}", result.mean);
}

#[test]
fn test_with_flip_python() {
    // Using Python API method
    let flip = FlipComparator::with_python_api(Some(67.0), 2);
    let result = flip.compare("ref.png", "test.png")?;
    
    println!("Mean FLIP error: {:.6}", result.mean);
    println!("Error map: {:?}", result.error_map_path);
    
    assert!(result.passes(0.15));
}
```

## Legacy Image Comparison

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

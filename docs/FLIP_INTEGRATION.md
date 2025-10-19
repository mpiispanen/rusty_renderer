# FLIP Integration Guide

This document describes the integration of NVIDIA FLIP (Feature-based Locally-Adaptive Pixel) perceptual image comparison into the rusty_renderer test suite.

## Overview

FLIP is an industry-standard perceptual image quality metric developed by NVIDIA that provides more accurate assessment of visual differences compared to traditional pixel-based metrics like MSE or PSNR. It accounts for:

- **Spatial frequency filtering** based on human contrast sensitivity
- **Color perception** in LMS (cone response) color space  
- **Spatial pooling** for local adaptation
- **Perceptually uniform difference maps**

## Implementation

### Dual-Method Architecture

We support two methods for FLIP comparison:

#### 1. CLI Method (Original)
Uses the `flip` command-line tool from the `flip-evaluator` package:

```rust
let flip = FlipComparator::default();
let result = flip.compare("reference.png", "test.png")?;
```

**Characteristics:**
- Parses text output from CLI tool
- Simple integration
- Works with existing tool installations
- Output format: text-based parsing

#### 2. Python API Method (Recommended)
Uses our custom Python wrapper (`scripts/flip_compare.py`) for direct API access:

```rust
let flip = FlipComparator::with_python_api(None, 2);
let result = flip.compare("reference.png", "test.png")?;
```

**Characteristics:**
- JSON-based output for reliable parsing
- Direct access to FLIP Python API
- Detailed error maps with better control
- Better error handling and debugging
- Cleaner integration with automated tests

### Python Script Features

The `scripts/flip_compare.py` provides:

1. **Direct API Access**: Bypasses CLI and uses `flip_evaluator` Python module directly
2. **JSON Output**: Structured data easy to parse from Rust
3. **Error Map Generation**: Saves perceptual difference maps with magma colormap
4. **Flexible Parameters**: PPD, verbosity, custom output paths
5. **Exit Codes**: 0 for pass (<0.15 mean error), 1 for fail, 2 for errors

### Rust Implementation

The `FlipComparator` in `src/testing/flip.rs` provides:

```rust
pub struct FlipComparator {
    pub pixels_per_degree: Option<f64>,  // Default: 67 (4K at 0.7m)
    pub verbosity: u8,                   // 0-2
    pub use_python_api: bool,            // Method selection
}

pub struct FlipResult {
    pub mean: f64,          // Mean FLIP error
    pub median: f64,        // Median error
    pub q1: f64,           // 1st quartile
    pub q3: f64,           // 3rd quartile
    pub min: f64,          // Minimum error
    pub max: f64,          // Maximum error
    pub ppd: f64,          // Pixels per degree used
    pub error_map_path: Option<String>,
}
```

## Installation

### Requirements

```bash
# Install FLIP evaluator and dependencies
pip install flip-evaluator numpy pillow

# Verify installation
flip --help
python3 scripts/flip_compare.py --help
```

### Optional: System-wide Installation

```bash
# Install for all users
sudo pip install flip-evaluator numpy pillow

# Or user-local installation
pip install --user flip-evaluator numpy pillow
```

## Usage Examples

### Rust Tests

```rust
#[test]
fn test_rendering_with_flip() {
    let output_dir = PathBuf::from("target/visual_tests");
    let ref_path = output_dir.join("reference.png");
    let test_path = output_dir.join("test.png");
    
    // Render images...
    
    // Compare with FLIP (Python API method)
    let flip = FlipComparator::with_python_api(None, 2);
    let result = flip.compare(&ref_path, &test_path)
        .expect("FLIP comparison failed");
    
    println!("Mean FLIP error: {:.6}", result.mean);
    
    // Assert within acceptable threshold
    assert!(result.passes(0.15), 
        "FLIP error too high: {:.6}", result.mean);
}
```

### Command Line

```bash
# Direct Python script usage
python3 scripts/flip_compare.py \
    target/visual_tests/vulkan_triangle.png \
    target/visual_tests/wgpu_triangle.png \
    --error-map diff.png \
    --output results.json

# Batch comparison
./scripts/batch_flip_compare.sh \
    reference_images/ \
    test_images/ \
    results/

# Run Rust tests with FLIP
cargo test --test visual_tests test_vulkan_vs_wgpu_flip -- --ignored --nocapture
```

## Thresholds and Interpretation

### Recommended Mean Error Thresholds

| Threshold | Interpretation | Use Case |
|-----------|----------------|----------|
| < 0.05    | Excellent match | Same hardware, same backend |
| < 0.10    | Good match | Different backends, expected variance |
| < 0.15    | Acceptable | Cross-platform, different rasterization |
| ≥ 0.15    | Significant differences | Investigate visual artifacts |

### Understanding Results

```json
{
  "mean": 0.081237,      // Primary metric (target < 0.15)
  "median": 0.001462,    // Most pixels have low error
  "q1": 0.001462,        // 25% of pixels below this
  "q3": 0.001462,        // 75% of pixels below this
  "min": 0.001462,       // Best case
  "max": 0.997351,       // Worst case (edges, anti-aliasing)
  "ppd": 67.0            // Viewing distance parameter
}
```

**Analysis:**
- Low median with higher mean suggests localized differences (edges, anti-aliasing)
- High max values are normal at geometric boundaries
- Q1/Q3 range indicates error distribution uniformity

## Backend Comparison Strategy

Different rendering backends produce slight variations due to:

1. **Rasterization Rules**: Edge handling, pixel centers
2. **Precision**: Float vs fixed-point calculations
3. **Coordinate Systems**: Y-up vs Y-down transformations
4. **Implementation**: Driver optimizations, hardware specifics

### Expected FLIP Errors by Backend Pair

| Comparison | Expected Mean | Notes |
|------------|---------------|-------|
| Vulkan vs Vulkan | < 0.001 | Identical backend |
| Vulkan vs DirectX | < 0.05 | Similar architecture |
| Vulkan vs wgpu | < 0.10 | Abstraction layer differences |
| wgpu vs DirectX | < 0.15 | Different underlying systems |

## Troubleshooting

### "flip command not found"
```bash
pip install flip-evaluator
# Add ~/.local/bin to PATH if needed
export PATH="$HOME/.local/bin:$PATH"
```

### "No module named 'numpy'"
```bash
pip install numpy
```

### "Python script not found"
Ensure you run tests from repository root:
```bash
cd /path/to/rusty_renderer
cargo test --test visual_tests
```

### High FLIP errors
1. Check error map visually: `error_map.png`
2. Verify images are correct size/format
3. Review rasterization differences documentation
4. Consider if threshold should be adjusted for use case

## Performance Considerations

### CLI Method
- **Speed**: ~200-500ms per comparison
- **Memory**: Low (external process)
- **Dependencies**: `flip` command only

### Python API Method  
- **Speed**: ~150-300ms per comparison
- **Memory**: Moderate (numpy arrays in memory)
- **Dependencies**: Python 3, numpy, flip-evaluator

Both methods are fast enough for automated testing. Python API method is recommended for CI/CD pipelines due to better error handling and JSON output.

## Integration with CI/CD

Example GitHub Actions workflow:

```yaml
# Linux: Test Vulkan and wgpu
- name: Install FLIP
  run: |
    pip install flip-evaluator numpy pillow

- name: Test Vulkan rendering
  run: |
    ./target/release/rusty_renderer \
      --backend vulkan --headless \
      --screenshot screenshots/vulkan-triangle.png --max-frames 1

- name: Test wgpu rendering
  run: |
    ./target/release/rusty_renderer \
      --backend wgpu --headless \
      --screenshot screenshots/wgpu-triangle.png --max-frames 1

# Windows: Test DirectX
- name: Test DirectX rendering
  run: |
    ./target/release/rusty_renderer.exe \
      --backend directx --headless \
      --screenshot screenshots/directx-triangle.png --max-frames 1

# Generate comprehensive report comparing all backends
- name: Generate visual regression report
  run: |
    python3 scripts/generate_visual_report.py \
      screenshots/ \
      visual-regression-report.html

- name: Upload Report
  uses: actions/upload-artifact@v4
  with:
    name: visual-regression-report
    path: |
      visual-regression-report.html
      screenshots/
      flip_results/
```

The actual CI workflow:
1. **Linux job** renders with Vulkan and wgpu
2. **Windows job** renders with DirectX 12
3. **Report job** downloads all screenshots and generates comprehensive HTML report
4. Report compares all backend pairs (Vulkan-wgpu, Vulkan-DirectX, wgpu-DirectX)
5. Uploads combined report as CI artifact

**Accessing Reports:**
- Go to GitHub Actions → Workflow Run
- Download `visual-regression-report-all-backends` artifact
- Open `visual-regression-report.html` in browser

## Future Enhancements

Potential improvements:

1. **HDR Support**: Extend to HDR-FLIP for tone-mapped content
2. **Batch Processing**: Parallel comparison of multiple image pairs
3. **Regression Tracking**: Historical FLIP error database
4. **Custom Pooling**: Scene-specific weighting for critical regions
5. **Native Rust**: Pure Rust FLIP implementation (significant effort)

## References

- [NVIDIA FLIP Paper](https://research.nvidia.com/publication/2020-07_FLIP)
- [flip-evaluator PyPI](https://pypi.org/project/flip-evaluator/)
- [FLIP GitHub Repository](https://github.com/NVlabs/flip)

## License Note

FLIP is developed by NVIDIA and distributed under BSD-3-Clause license. Our integration scripts maintain compatibility with this license.

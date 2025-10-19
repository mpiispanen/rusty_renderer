#!/usr/bin/env python3
"""
Compare test screenshots against baseline reference images.

This script compares newly generated screenshots against known-good baseline
images stored in the references/ directory. It generates a comprehensive HTML
report and exits with failure if any comparison exceeds the threshold.

Usage:
    python3 compare_against_baseline.py references/triangle/ test-output/ report.html
    python3 compare_against_baseline.py references/triangle/ test-output/ report.html --threshold 0.10
"""

import argparse
import base64
import json
import sys
from pathlib import Path
from datetime import datetime

try:
    import flip_evaluator
    import numpy as np
except ImportError as e:
    print(f"Error: Required package not installed: {e}", file=sys.stderr)
    print("Install with: pip install flip-evaluator numpy", file=sys.stderr)
    sys.exit(2)


def image_to_base64(image_path):
    """Convert image to base64 data URI."""
    with open(image_path, 'rb') as f:
        data = base64.b64encode(f.read()).decode('utf-8')
    return f"data:image/png;base64,{data}"


def compare_images(ref_path, test_path, output_dir):
    """Compare two images using FLIP and return results."""
    ref_path = Path(ref_path)
    test_path = Path(test_path)
    output_dir = Path(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Determine dynamic range
    ext = ref_path.suffix.lower()
    dynamic_range = "HDR" if ext == ".exr" else "LDR"
    
    # Run FLIP
    error_map, mean_error, used_parameters = flip_evaluator.evaluate(
        str(ref_path),
        str(test_path),
        dynamic_range,
        inputsRGB=True,
        applyMagma=True,
        computeMeanError=True,
        parameters={}
    )
    
    # Calculate statistics
    error_values = error_map[:, :, 0]
    flat_errors = error_values.flatten()
    
    # Save error map
    error_map_path = output_dir / f"error_{ref_path.stem}_vs_{test_path.stem}.png"
    try:
        from PIL import Image
        error_map_8bit = (error_map * 255).astype(np.uint8)
        img = Image.fromarray(error_map_8bit, mode='RGB')
        img.save(error_map_path)
    except ImportError:
        print("Warning: PIL not available, cannot save error map", file=sys.stderr)
        error_map_path = None
    
    return {
        "mean": float(mean_error),
        "median": float(np.median(flat_errors)),
        "max": float(np.max(flat_errors)),
        "min": float(np.min(flat_errors)),
        "ppd": float(used_parameters.get("ppd", 67.0)),
        "error_map_path": error_map_path,
        "reference": ref_path,
        "test": test_path,
    }


def generate_html_report(comparisons, output_path, threshold):
    """Generate HTML report for baseline comparison."""
    
    passed = sum(1 for c in comparisons if c["result"]["mean"] < threshold)
    total = len(comparisons)
    
    # Generate comparison sections
    comparisons_html = ""
    for comp in comparisons:
        result = comp["result"]
        mean_error = result["mean"]
        
        # Determine status
        if mean_error < threshold:
            status_class = "pass"
            status_text = "PASS"
            comparison_class = ""
        else:
            status_class = "fail"
            status_text = "FAIL"
            comparison_class = "error"
        
        # Generate metrics
        metrics_html = f"""
            <div class="metric">
                <div class="metric-label">Mean Error</div>
                <div class="metric-value">{mean_error:.6f}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Threshold</div>
                <div class="metric-value">{threshold:.2f}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Median</div>
                <div class="metric-value">{result['median']:.6f}</div>
            </div>
            <div class="metric">
                <div class="metric-label">Max</div>
                <div class="metric-value">{result['max']:.6f}</div>
            </div>
        """
        
        # Generate images
        ref_b64 = image_to_base64(comp["reference"])
        test_b64 = image_to_base64(comp["test"])
        
        images_html = f"""
            <div class="image-container">
                <h4>Reference (Baseline)</h4>
                <img src="{ref_b64}" alt="Reference">
            </div>
            <div class="image-container">
                <h4>Test (Current)</h4>
                <img src="{test_b64}" alt="Test">
            </div>
        """
        
        if result.get("error_map_path") and result["error_map_path"].exists():
            error_b64 = image_to_base64(result["error_map_path"])
            images_html += f"""
            <div class="image-container">
                <h4>FLIP Error Map</h4>
                <img src="{error_b64}" alt="Error Map">
            </div>
            """
        
        comparisons_html += f"""
        <div class="comparison {comparison_class}">
            <h2>
                {comp["name"]}
                <span class="status-badge {status_class}">{status_text}</span>
            </h2>
            <div class="metrics">
                {metrics_html}
            </div>
            <div class="images">
                {images_html}
            </div>
        </div>
        """
    
    # Generate HTML
    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Baseline Comparison Report</title>
    <style>
        * {{ margin: 0; padding: 0; box-sizing: border-box; }}
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; line-height: 1.6; color: #333; background: #f5f5f5; padding: 20px; }}
        .container {{ max-width: 1400px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 8px rgba(0,0,0,0.1); }}
        header {{ border-bottom: 3px solid #4CAF50; padding-bottom: 20px; margin-bottom: 30px; }}
        h1 {{ color: #2c3e50; font-size: 2.5em; margin-bottom: 10px; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }}
        .summary-card {{ background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 20px; border-radius: 8px; }}
        .summary-card h3 {{ font-size: 0.9em; opacity: 0.9; margin-bottom: 10px; }}
        .summary-card .value {{ font-size: 2em; font-weight: bold; }}
        .comparison {{ margin-bottom: 50px; padding: 25px; background: #fafafa; border-radius: 8px; border-left: 4px solid #4CAF50; }}
        .comparison.error {{ border-left-color: #f44336; }}
        .comparison h2 {{ color: #2c3e50; margin-bottom: 20px; display: flex; align-items: center; gap: 10px; }}
        .status-badge {{ padding: 4px 12px; border-radius: 4px; font-size: 0.85em; font-weight: bold; }}
        .status-badge.pass {{ background: #4CAF50; color: white; }}
        .status-badge.fail {{ background: #f44336; color: white; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 15px; margin-bottom: 25px; }}
        .metric {{ background: white; padding: 15px; border-radius: 6px; box-shadow: 0 2px 4px rgba(0,0,0,0.05); }}
        .metric-label {{ font-size: 0.85em; color: #666; margin-bottom: 5px; }}
        .metric-value {{ font-size: 1.5em; font-weight: bold; color: #2c3e50; }}
        .images {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin-top: 25px; }}
        .image-container {{ background: white; padding: 15px; border-radius: 6px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .image-container h4 {{ margin-bottom: 10px; color: #2c3e50; font-size: 0.95em; }}
        .image-container img {{ width: 100%; height: auto; border-radius: 4px; border: 1px solid #ddd; }}
        footer {{ margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; text-align: center; color: #666; font-size: 0.9em; }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Baseline Comparison Report</h1>
            <p><strong>Generated:</strong> {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}</p>
            <p><strong>Threshold:</strong> {threshold:.2f} mean FLIP error</p>
        </header>
        
        <div class="summary">
            <div class="summary-card">
                <h3>Total Tests</h3>
                <div class="value">{total}</div>
            </div>
            <div class="summary-card">
                <h3>Passed</h3>
                <div class="value">{passed}</div>
            </div>
            <div class="summary-card">
                <h3>Failed</h3>
                <div class="value">{total - passed}</div>
            </div>
            <div class="summary-card">
                <h3>Pass Rate</h3>
                <div class="value">{(passed/total*100) if total > 0 else 0:.0f}%</div>
            </div>
        </div>
        
        {comparisons_html}
        
        <footer>
            <p>Generated by rusty_renderer baseline comparison</p>
            <p>Using NVIDIA FLIP for perceptual image comparison</p>
        </footer>
    </div>
</body>
</html>
"""
    
    output_path.write_text(html)
    return passed == total


def main():
    parser = argparse.ArgumentParser(
        description="Compare test screenshots against baseline references"
    )
    parser.add_argument(
        "reference_dir",
        type=Path,
        help="Directory containing reference images"
    )
    parser.add_argument(
        "test_dir",
        type=Path,
        help="Directory containing test screenshots"
    )
    parser.add_argument(
        "output_html",
        type=Path,
        help="Output HTML report path"
    )
    parser.add_argument(
        "--threshold",
        type=float,
        default=0.10,
        help="FLIP error threshold (default: 0.10)"
    )
    parser.add_argument(
        "--temp-dir",
        type=Path,
        default=Path("baseline_results"),
        help="Temporary directory for FLIP results"
    )
    
    args = parser.parse_args()
    
    if not args.reference_dir.exists():
        print(f"Error: Reference directory not found: {args.reference_dir}", file=sys.stderr)
        sys.exit(2)
    
    if not args.test_dir.exists():
        print(f"Error: Test directory not found: {args.test_dir}", file=sys.stderr)
        sys.exit(2)
    
    # Find reference images
    references = {}
    for png_file in args.reference_dir.glob("*.png"):
        name = png_file.stem
        references[name] = png_file
    
    if not references:
        print(f"Error: No reference images found in {args.reference_dir}", file=sys.stderr)
        sys.exit(2)
    
    print(f"Found {len(references)} reference image(s)")
    
    # Compare each reference against corresponding test image
    comparisons = []
    missing_tests = []
    
    for name, ref_path in references.items():
        # Look for corresponding test image
        test_path = args.test_dir / ref_path.name
        if not test_path.exists():
            missing_tests.append(name)
            continue
        
        print(f"Comparing {name}...")
        result = compare_images(ref_path, test_path, args.temp_dir)
        
        comparisons.append({
            "name": name,
            "reference": ref_path,
            "test": test_path,
            "result": result,
        })
    
    if missing_tests:
        print(f"\nWarning: No test images found for: {', '.join(missing_tests)}", file=sys.stderr)
    
    if not comparisons:
        print("Error: No comparisons performed (no matching test images)", file=sys.stderr)
        sys.exit(2)
    
    # Generate report
    all_passed = generate_html_report(comparisons, args.output_html, args.threshold)
    
    # Print summary
    passed = sum(1 for c in comparisons if c["result"]["mean"] < args.threshold)
    total = len(comparisons)
    
    print(f"\n{'='*60}")
    print(f"Baseline Comparison Results")
    print(f"{'='*60}")
    print(f"Total comparisons: {total}")
    print(f"Passed: {passed}/{total}")
    print(f"Threshold: {args.threshold:.2f} mean FLIP error")
    print(f"Report: {args.output_html}")
    
    if not all_passed:
        print(f"\n❌ FAILED: {total - passed} comparison(s) exceeded threshold")
        for comp in comparisons:
            if comp["result"]["mean"] >= args.threshold:
                print(f"  - {comp['name']}: {comp['result']['mean']:.6f} (threshold: {args.threshold:.2f})")
        sys.exit(1)
    else:
        print(f"\n✅ All comparisons passed!")
        sys.exit(0)


if __name__ == "__main__":
    main()

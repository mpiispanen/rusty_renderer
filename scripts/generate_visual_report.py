#!/usr/bin/env python3
"""
Generate HTML visual regression report comparing multiple backend outputs.

This script compares screenshots from different backends using FLIP and generates
a comprehensive HTML report with embedded images, FLIP metrics, and error maps.

Usage:
    python3 generate_visual_report.py screenshots/ output_report.html
"""

import argparse
import base64
import json
import sys
from pathlib import Path
from datetime import datetime
import subprocess

try:
    import flip_evaluator
    import numpy as np
except ImportError as e:
    print(f"Error: Required package not installed: {e}", file=sys.stderr)
    print("Install with: pip install flip-evaluator numpy", file=sys.stderr)
    sys.exit(1)


HTML_TEMPLATE = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Visual Regression Report - {title}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }}
        
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }}
        
        header {{
            border-bottom: 3px solid #4CAF50;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        
        h1 {{
            color: #2c3e50;
            font-size: 2.5em;
            margin-bottom: 10px;
        }}
        
        .meta-info {{
            color: #666;
            font-size: 0.95em;
        }}
        
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}
        
        .summary-card {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        
        .summary-card h3 {{
            font-size: 0.9em;
            opacity: 0.9;
            margin-bottom: 10px;
        }}
        
        .summary-card .value {{
            font-size: 2em;
            font-weight: bold;
        }}
        
        .comparison {{
            margin-bottom: 50px;
            padding: 25px;
            background: #fafafa;
            border-radius: 8px;
            border-left: 4px solid #4CAF50;
        }}
        
        .comparison.warning {{
            border-left-color: #ff9800;
        }}
        
        .comparison.error {{
            border-left-color: #f44336;
        }}
        
        .comparison h2 {{
            color: #2c3e50;
            margin-bottom: 20px;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        
        .status-badge {{
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.85em;
            font-weight: bold;
        }}
        
        .status-badge.pass {{
            background: #4CAF50;
            color: white;
        }}
        
        .status-badge.warning {{
            background: #ff9800;
            color: white;
        }}
        
        .status-badge.fail {{
            background: #f44336;
            color: white;
        }}
        
        .metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
            margin-bottom: 25px;
        }}
        
        .metric {{
            background: white;
            padding: 15px;
            border-radius: 6px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
        }}
        
        .metric-label {{
            font-size: 0.85em;
            color: #666;
            margin-bottom: 5px;
        }}
        
        .metric-value {{
            font-size: 1.5em;
            font-weight: bold;
            color: #2c3e50;
        }}
        
        .images {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
            margin-top: 25px;
        }}
        
        .image-container {{
            background: white;
            padding: 15px;
            border-radius: 6px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        
        .image-container h4 {{
            margin-bottom: 10px;
            color: #2c3e50;
            font-size: 0.95em;
        }}
        
        .image-container img {{
            width: 100%;
            height: auto;
            border-radius: 4px;
            border: 1px solid #ddd;
        }}
        
        .interpretation {{
            margin-top: 15px;
            padding: 15px;
            background: white;
            border-radius: 6px;
            border-left: 3px solid #2196F3;
        }}
        
        .interpretation h4 {{
            color: #2c3e50;
            margin-bottom: 10px;
        }}
        
        footer {{
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            text-align: center;
            color: #666;
            font-size: 0.9em;
        }}
        
        .threshold-info {{
            background: #e3f2fd;
            padding: 15px;
            border-radius: 6px;
            margin-bottom: 25px;
            border-left: 3px solid #2196F3;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Visual Regression Report</h1>
            <div class="meta-info">
                <p><strong>Generated:</strong> {timestamp}</p>
                <p><strong>Backends Compared:</strong> {backends}</p>
            </div>
        </header>
        
        <div class="summary">
            {summary_cards}
        </div>
        
        <div class="threshold-info">
            <strong>FLIP Thresholds:</strong>
            <ul style="margin-left: 20px; margin-top: 5px;">
                <li>&lt; 0.05: Excellent match (imperceptible differences)</li>
                <li>&lt; 0.10: Good match (minor differences)</li>
                <li>&lt; 0.15: Acceptable match (noticeable but acceptable)</li>
                <li>≥ 0.15: Significant differences (investigation needed)</li>
            </ul>
        </div>
        
        {comparisons}
        
        <footer>
            <p>Generated by rusty_renderer visual regression testing</p>
            <p>Using NVIDIA FLIP for perceptual image comparison</p>
        </footer>
    </div>
</body>
</html>
"""


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
        "q1": float(np.percentile(flat_errors, 25)),
        "q3": float(np.percentile(flat_errors, 75)),
        "min": float(np.min(flat_errors)),
        "max": float(np.max(flat_errors)),
        "ppd": float(used_parameters.get("ppd", 67.0)),
        "error_map_path": error_map_path,
        "reference": ref_path,
        "test": test_path,
    }


def generate_comparison_html(comparison_name, ref_img, test_img, result):
    """Generate HTML for a single comparison."""
    mean_error = result["mean"]
    
    # Determine status
    if mean_error < 0.05:
        status = "pass"
        status_text = "EXCELLENT"
        comparison_class = ""
    elif mean_error < 0.10:
        status = "pass"
        status_text = "GOOD"
        comparison_class = ""
    elif mean_error < 0.15:
        status = "warning"
        status_text = "ACCEPTABLE"
        comparison_class = "warning"
    else:
        status = "fail"
        status_text = "FAIL"
        comparison_class = "error"
    
    # Interpretation
    if mean_error < 0.05:
        interpretation = "Images are visually identical or have imperceptible differences."
    elif mean_error < 0.10:
        interpretation = "Images have minor differences that are barely noticeable."
    elif mean_error < 0.15:
        interpretation = "Images have noticeable but acceptable differences, likely due to rasterization or precision variations."
    else:
        interpretation = "Images have significant differences that require investigation."
    
    # Generate metrics HTML
    metrics_html = f"""
        <div class="metric">
            <div class="metric-label">Mean Error</div>
            <div class="metric-value">{mean_error:.6f}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Median</div>
            <div class="metric-value">{result['median']:.6f}</div>
        </div>
        <div class="metric">
            <div class="metric-label">Max Error</div>
            <div class="metric-value">{result['max']:.6f}</div>
        </div>
        <div class="metric">
            <div class="metric-label">PPD</div>
            <div class="metric-value">{result['ppd']:.1f}</div>
        </div>
    """
    
    # Generate images HTML
    ref_b64 = image_to_base64(ref_img)
    test_b64 = image_to_base64(test_img)
    
    images_html = f"""
        <div class="image-container">
            <h4>{Path(ref_img).stem}</h4>
            <img src="{ref_b64}" alt="Reference">
        </div>
        <div class="image-container">
            <h4>{Path(test_img).stem}</h4>
            <img src="{test_b64}" alt="Test">
        </div>
    """
    
    # Add error map if available
    if result.get("error_map_path") and result["error_map_path"].exists():
        error_b64 = image_to_base64(result["error_map_path"])
        images_html += f"""
        <div class="image-container">
            <h4>FLIP Error Map</h4>
            <img src="{error_b64}" alt="Error Map">
        </div>
        """
    
    return f"""
    <div class="comparison {comparison_class}">
        <h2>
            {comparison_name}
            <span class="status-badge {status}">{status_text}</span>
        </h2>
        <div class="metrics">
            {metrics_html}
        </div>
        <div class="images">
            {images_html}
        </div>
        <div class="interpretation">
            <h4>Interpretation:</h4>
            <p>{interpretation}</p>
        </div>
    </div>
    """


def main():
    parser = argparse.ArgumentParser(
        description="Generate HTML visual regression report"
    )
    parser.add_argument(
        "screenshot_dir",
        help="Directory containing backend screenshots"
    )
    parser.add_argument(
        "output_html",
        help="Output HTML report path"
    )
    parser.add_argument(
        "--temp-dir",
        default="flip_temp",
        help="Temporary directory for FLIP results"
    )
    
    args = parser.parse_args()
    
    screenshot_dir = Path(args.screenshot_dir)
    output_html = Path(args.output_html)
    temp_dir = Path(args.temp_dir)
    
    if not screenshot_dir.exists():
        print(f"Error: Screenshot directory not found: {screenshot_dir}", file=sys.stderr)
        sys.exit(1)
    
    # Find all backend screenshots
    backends = {}
    for png_file in screenshot_dir.glob("*triangle*.png"):
        name = png_file.stem
        if "vulkan" in name.lower():
            backends["vulkan"] = png_file
        elif "wgpu" in name.lower():
            backends["wgpu"] = png_file
        elif "directx" in name.lower() or "dx" in name.lower():
            backends["directx"] = png_file
    
    if len(backends) < 2:
        print(f"Error: Need at least 2 backend screenshots, found {len(backends)}", file=sys.stderr)
        sys.exit(1)
    
    print(f"Found backends: {', '.join(backends.keys())}")
    
    # Perform comparisons
    comparisons = []
    backend_list = list(backends.items())
    
    for i in range(len(backend_list)):
        for j in range(i + 1, len(backend_list)):
            name1, img1 = backend_list[i]
            name2, img2 = backend_list[j]
            
            print(f"Comparing {name1} vs {name2}...")
            result = compare_images(img1, img2, temp_dir)
            
            comparisons.append({
                "name": f"{name1.capitalize()} vs {name2.capitalize()}",
                "ref_img": img1,
                "test_img": img2,
                "result": result,
            })
    
    # Generate summary
    total_comparisons = len(comparisons)
    passed = sum(1 for c in comparisons if c["result"]["mean"] < 0.15)
    excellent = sum(1 for c in comparisons if c["result"]["mean"] < 0.05)
    
    summary_cards = f"""
        <div class="summary-card">
            <h3>Total Comparisons</h3>
            <div class="value">{total_comparisons}</div>
        </div>
        <div class="summary-card">
            <h3>Passed (< 0.15)</h3>
            <div class="value">{passed}</div>
        </div>
        <div class="summary-card">
            <h3>Excellent (< 0.05)</h3>
            <div class="value">{excellent}</div>
        </div>
        <div class="summary-card">
            <h3>Backends</h3>
            <div class="value">{len(backends)}</div>
        </div>
    """
    
    # Generate comparison sections
    comparisons_html = ""
    for comp in comparisons:
        comparisons_html += generate_comparison_html(
            comp["name"],
            comp["ref_img"],
            comp["test_img"],
            comp["result"]
        )
    
    # Generate final HTML
    html = HTML_TEMPLATE.format(
        title="Backend Comparison",
        timestamp=datetime.now().strftime("%Y-%m-%d %H:%M:%S"),
        backends=", ".join(b.capitalize() for b in backends.keys()),
        summary_cards=summary_cards,
        comparisons=comparisons_html,
    )
    
    # Write output
    output_html.parent.mkdir(parents=True, exist_ok=True)
    output_html.write_text(html)
    
    print(f"\n✅ Report generated: {output_html}")
    print(f"   Total comparisons: {total_comparisons}")
    print(f"   Passed: {passed}/{total_comparisons}")
    
    # Exit code based on results
    sys.exit(0 if passed == total_comparisons else 1)


if __name__ == "__main__":
    main()

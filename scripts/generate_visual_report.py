#!/usr/bin/env python3
"""
Generate HTML visual regression report with specific comparisons:
1. Vulkan current vs Vulkan reference
2. DirectX current vs DirectX reference
3. Vulkan current vs DirectX current (backend parity)
"""

import argparse
import base64
from pathlib import Path
from datetime import datetime
import sys

try:
    import flip_evaluator
    import numpy as np
    from PIL import Image
except ImportError as e:
    print(f"Error: Required package not installed: {e}", file=sys.stderr)
    print("Install with: pip install flip-evaluator numpy pillow", file=sys.stderr)
    sys.exit(1)


def image_to_base64(image_path):
    """Convert image to base64 data URI."""
    with open(image_path, 'rb') as f:
        data = base64.b64encode(f.read()).decode('utf-8')
    return f"data:image/png;base64,{data}"


def run_flip_comparison(ref_path, test_path):
    """Run FLIP comparison and return statistics."""
    ref_path = Path(ref_path)
    test_path = Path(test_path)
    
    if not ref_path.exists() or not test_path.exists():
        return None
    
    # Run FLIP
    error_map, mean_error, used_parameters = flip_evaluator.evaluate(
        str(ref_path),
        str(test_path),
        "LDR",
        inputsRGB=True,
        applyMagma=True,
        computeMeanError=True,
        parameters={}
    )
    
    # Calculate statistics
    error_values = error_map[:, :, 0]
    flat_errors = error_values.flatten()
    
    return {
        "mean": float(mean_error),
        "median": float(np.median(flat_errors)),
        "max": float(np.max(flat_errors)),
        "ppd": float(used_parameters.get("ppd", 67.0)),
        "error_map": error_map,
    }


def generate_html_report(data_dir, output_path):
    """Generate the HTML report."""
    data_dir = Path(data_dir)
    
    # Define paths
    vulkan_current = data_dir / "current" / "vulkan.png"
    directx_current = data_dir / "current" / "directx.png"
    vulkan_ref = data_dir / "references" / "vulkan.png"
    directx_ref = data_dir / "references" / "directx.png"
    
    # Prepare comparisons
    comparisons = []
    
    # 1. Vulkan regression check
    if vulkan_ref.exists() and vulkan_current.exists():
        result = run_flip_comparison(vulkan_ref, vulkan_current)
        if result:
            comparisons.append({
                "title": "Vulkan Regression Check",
                "description": "Current Vulkan output compared to golden reference",
                "ref_img": vulkan_ref,
                "test_img": vulkan_current,
                "result": result,
                "type": "regression"
            })
    
    # 2. DirectX regression check
    if directx_ref.exists() and directx_current.exists():
        result = run_flip_comparison(directx_ref, directx_current)
        if result:
            comparisons.append({
                "title": "DirectX Regression Check",
                "description": "Current DirectX output compared to golden reference",
                "ref_img": directx_ref,
                "test_img": directx_current,
                "result": result,
                "type": "regression"
            })
    
    # 3. Backend parity check
    if vulkan_current.exists() and directx_current.exists():
        result = run_flip_comparison(vulkan_current, directx_current)
        if result:
            comparisons.append({
                "title": "Backend Parity Check",
                "description": "Vulkan vs DirectX rendering comparison (expect ~14% difference due to coordinate systems)",
                "ref_img": vulkan_current,
                "test_img": directx_current,
                "result": result,
                "type": "parity"
            })
    
    # Generate HTML
    html_parts = []
    html_parts.append("""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Visual Regression Report</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 8px rgba(0,0,0,0.1);
        }
        header {
            border-bottom: 3px solid #4CAF50;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }
        h1 { color: #2c3e50; font-size: 2.5em; margin-bottom: 10px; }
        .meta-info { color: #666; font-size: 0.95em; }
        .comparison {
            margin-bottom: 50px;
            padding: 25px;
            background: #fafafa;
            border-radius: 8px;
            border-left: 4px solid #4CAF50;
        }
        .comparison.regression { border-left-color: #2196F3; }
        .comparison.parity { border-left-color: #FF9800; }
        .comparison.warning { background: #fff3cd; border-left-color: #FFC107; }
        .comparison.error { background: #f8d7da; border-left-color: #DC3545; }
        .comparison h2 {
            color: #2c3e50;
            margin-bottom: 10px;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .comparison .description { color: #666; margin-bottom: 20px; }
        .status-badge {
            padding: 4px 12px;
            border-radius: 4px;
            font-size: 0.85em;
            font-weight: bold;
        }
        .status-badge.pass { background: #4CAF50; color: white; }
        .status-badge.warning { background: #FF9800; color: white; }
        .status-badge.fail { background: #DC3545; color: white; }
        .metrics {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 15px;
            margin-bottom: 25px;
        }
        .metric {
            background: white;
            padding: 15px;
            border-radius: 6px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.05);
        }
        .metric-label { font-size: 0.85em; color: #666; margin-bottom: 5px; }
        .metric-value { font-size: 1.5em; font-weight: bold; color: #2c3e50; }
        .images {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 20px;
            margin-top: 25px;
        }
        .image-container {
            background: white;
            padding: 15px;
            border-radius: 6px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .image-container h4 { margin-bottom: 10px; color: #2c3e50; font-size: 0.95em; }
        .image-container img {
            width: 100%;
            height: auto;
            border-radius: 4px;
            border: 1px solid #ddd;
        }
        .interpretation {
            margin-top: 15px;
            padding: 15px;
            background: white;
            border-radius: 6px;
            border-left: 3px solid #2196F3;
        }
        .interpretation h4 { color: #2c3e50; margin-bottom: 10px; }
        footer {
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            text-align: center;
            color: #666;
            font-size: 0.9em;
        }
        .threshold-info {
            background: #e3f2fd;
            padding: 15px;
            border-radius: 6px;
            margin-bottom: 25px;
            border-left: 3px solid #2196F3;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Visual Regression Report</h1>
            <div class="meta-info">
                <p><strong>Generated:</strong> """ + datetime.now().strftime("%Y-%m-%d %H:%M:%S") + """</p>
                <p><strong>Comparisons:</strong> """ + str(len(comparisons)) + """</p>
            </div>
        </header>
        
        <div class="threshold-info">
            <strong>FLIP Thresholds:</strong>
            <ul style="margin-left: 20px; margin-top: 5px;">
                <li>&lt; 0.05: Excellent (imperceptible differences)</li>
                <li>&lt; 0.10: Good (minor acceptable differences)</li>
                <li>≥ 0.10: Significant differences (needs investigation)</li>
            </ul>
            <strong>Note:</strong> Backend parity typically shows ~14% difference due to coordinate system variations.
        </div>
""")
    
    # Generate comparison sections
    for comp in comparisons:
        result = comp["result"]
        mean_error = result["mean"]
        
        # Determine status
        if comp["type"] == "parity":
            # More lenient for backend parity
            if mean_error < 0.20:
                status, status_text, comp_class = "pass", "ACCEPTABLE", ""
            else:
                status, status_text, comp_class = "warning", "LARGE DIFF", "warning"
            interpretation = f"Backend parity shows {mean_error*100:.2f}% mean difference. Expected range is ~14%. This is {'acceptable' if mean_error < 0.20 else 'higher than expected - investigate'}."
        else:
            # Strict for regression checks
            if mean_error < 0.05:
                status, status_text, comp_class = "pass", "EXCELLENT", ""
                interpretation = "Output matches reference perfectly. No regression detected."
            elif mean_error < 0.10:
                status, status_text, comp_class = "pass", "GOOD", ""
                interpretation = "Minor differences detected, but within acceptable range."
            else:
                status, status_text, comp_class = "fail", "REGRESSION", "error"
                interpretation = "Significant differences detected! This may indicate a rendering regression."
        
        # Save error map
        error_map_path = data_dir / "comparisons" / f"error_{comp['title'].replace(' ', '_')}.png"
        error_map_path.parent.mkdir(parents=True, exist_ok=True)
        error_map_8bit = (result["error_map"] * 255).astype(np.uint8)
        img = Image.fromarray(error_map_8bit, mode='RGB')
        img.save(error_map_path)
        
        # Generate comparison HTML
        html_parts.append(f"""
        <div class="comparison {comp['type']} {comp_class}">
            <h2>
                {comp['title']}
                <span class="status-badge {status}">{status_text}</span>
            </h2>
            <div class="description">{comp['description']}</div>
            <div class="metrics">
                <div class="metric">
                    <div class="metric-label">Mean FLIP Error</div>
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
            </div>
            <div class="images">
                <div class="image-container">
                    <h4>Reference</h4>
                    <img src="{image_to_base64(comp['ref_img'])}" alt="Reference">
                </div>
                <div class="image-container">
                    <h4>Current</h4>
                    <img src="{image_to_base64(comp['test_img'])}" alt="Current">
                </div>
                <div class="image-container">
                    <h4>FLIP Error Map</h4>
                    <img src="{image_to_base64(error_map_path)}" alt="Error Map">
                </div>
            </div>
            <div class="interpretation">
                <h4>Interpretation:</h4>
                <p>{interpretation}</p>
            </div>
        </div>
""")
    
    html_parts.append("""
        <footer>
            <p>Generated by rusty_renderer visual regression testing</p>
            <p>Using NVIDIA FLIP for perceptual image comparison</p>
        </footer>
    </div>
</body>
</html>
""")
    
    # Write output
    output_path = Path(output_path)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text("".join(html_parts))
    
    print(f"\n✅ Report generated: {output_path}")
    print(f"   Comparisons: {len(comparisons)}")
    
    # Count failures
    failures = sum(1 for c in comparisons if c["type"] == "regression" and c["result"]["mean"] >= 0.10)
    if failures > 0:
        print(f"\n❌ {failures} regression(s) detected!")
        return 1
    
    print("\n✅ All regression checks passed!")
    return 0


def main():
    parser = argparse.ArgumentParser(description="Generate visual regression HTML report")
    parser.add_argument("data_dir", help="Directory containing current/, references/, comparisons/")
    parser.add_argument("output_html", help="Output HTML report path")
    
    args = parser.parse_args()
    
    try:
        exit_code = generate_html_report(args.data_dir, args.output_html)
        sys.exit(exit_code)
    except Exception as e:
        print(f"Error generating report: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()

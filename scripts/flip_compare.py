#!/usr/bin/env python3
"""
Python wrapper for FLIP image comparison using the flip-evaluator API.

This script provides a more direct interface to the FLIP Python API,
avoiding the need to parse command-line output. It can be called from
Rust tests to perform perceptual image comparison.

Usage:
    python flip_compare.py reference.png test.png [--ppd 67] [--output results.json]
"""

import argparse
import json
import sys
from pathlib import Path

try:
    import flip_evaluator
    import numpy as np
except ImportError as e:
    print(f"Error: Required package not installed: {e}", file=sys.stderr)
    print("Install with: pip install flip-evaluator numpy", file=sys.stderr)
    sys.exit(1)


def compare_images(reference_path, test_path, ppd=None, output_map=None, verbosity=2):
    """
    Compare two images using FLIP and return detailed metrics.
    
    Args:
        reference_path: Path to reference image
        test_path: Path to test image
        ppd: Pixels per degree (optional, uses FLIP default if None)
        output_map: Path to save error map (optional)
        verbosity: Verbosity level (0-2)
    
    Returns:
        Dictionary containing FLIP metrics and metadata
    """
    # Validate inputs
    reference_path = Path(reference_path)
    test_path = Path(test_path)
    
    if not reference_path.exists():
        raise FileNotFoundError(f"Reference image not found: {reference_path}")
    if not test_path.exists():
        raise FileNotFoundError(f"Test image not found: {test_path}")
    
    # Determine dynamic range based on file extension
    ext = reference_path.suffix.lower()
    dynamic_range = "HDR" if ext == ".exr" else "LDR"
    
    # Build parameters
    parameters = {}
    if ppd is not None:
        parameters["ppd"] = float(ppd)
    
    if verbosity >= 1:
        print(f"Comparing images using FLIP ({dynamic_range}):", file=sys.stderr)
        print(f"  Reference: {reference_path}", file=sys.stderr)
        print(f"  Test: {test_path}", file=sys.stderr)
        if ppd:
            print(f"  PPD: {ppd}", file=sys.stderr)
    
    # Perform FLIP evaluation
    error_map, mean_error, used_parameters = flip_evaluator.evaluate(
        str(reference_path),
        str(test_path),
        dynamic_range,
        inputsRGB=True,
        applyMagma=True,
        computeMeanError=True,
        parameters=parameters
    )
    
    # Calculate additional statistics from error map
    error_values = error_map[:, :, 0]  # FLIP error is in first channel
    
    # Calculate pooled statistics (weighted by error values)
    flat_errors = error_values.flatten()
    flat_errors_sorted = np.sort(flat_errors)
    
    # Weighted quartiles (FLIP uses weighted percentiles)
    # For simplicity, we use standard percentiles here
    q1 = np.percentile(flat_errors, 25)
    median = np.median(flat_errors)
    q3 = np.percentile(flat_errors, 75)
    min_error = np.min(flat_errors)
    max_error = np.max(flat_errors)
    
    # Build result dictionary
    result = {
        "mean": float(mean_error),
        "median": float(median),
        "q1": float(q1),
        "q3": float(q3),
        "min": float(min_error),
        "max": float(max_error),
        "ppd": float(used_parameters.get("ppd", 67.0)),
        "dynamic_range": dynamic_range,
        "reference": str(reference_path),
        "test": str(test_path),
    }
    
    # Save error map if requested
    if output_map:
        output_path = Path(output_map)
        # Error map is already in magma colormap (RGB)
        try:
            from PIL import Image
            # Convert to 8-bit RGB
            error_map_8bit = (error_map * 255).astype(np.uint8)
            img = Image.fromarray(error_map_8bit, mode='RGB')
            img.save(output_path)
            result["error_map"] = str(output_path)
            if verbosity >= 2:
                print(f"  Error map saved: {output_path}", file=sys.stderr)
        except ImportError:
            if verbosity >= 1:
                print("  Warning: PIL not available, cannot save error map", file=sys.stderr)
    
    if verbosity >= 2:
        print(f"\nFLIP Results:", file=sys.stderr)
        print(f"  Mean error: {result['mean']:.6f}", file=sys.stderr)
        print(f"  Median: {result['median']:.6f}", file=sys.stderr)
        print(f"  Q1: {result['q1']:.6f}", file=sys.stderr)
        print(f"  Q3: {result['q3']:.6f}", file=sys.stderr)
        print(f"  Min: {result['min']:.6f}", file=sys.stderr)
        print(f"  Max: {result['max']:.6f}", file=sys.stderr)
        print(f"  PPD: {result['ppd']:.1f}", file=sys.stderr)
    
    return result


def main():
    parser = argparse.ArgumentParser(
        description="Compare two images using NVIDIA FLIP perceptual metric"
    )
    parser.add_argument(
        "reference",
        help="Path to reference image (PNG or EXR)"
    )
    parser.add_argument(
        "test",
        help="Path to test image (PNG or EXR)"
    )
    parser.add_argument(
        "--ppd",
        type=float,
        help="Pixels per degree (default: 67 for 4K at 0.7m)"
    )
    parser.add_argument(
        "--output",
        "-o",
        help="Path to save JSON results"
    )
    parser.add_argument(
        "--error-map",
        "-e",
        help="Path to save error map image"
    )
    parser.add_argument(
        "--verbosity",
        "-v",
        type=int,
        default=2,
        choices=[0, 1, 2],
        help="Verbosity level (0=silent, 1=basic, 2=detailed)"
    )
    
    args = parser.parse_args()
    
    try:
        result = compare_images(
            args.reference,
            args.test,
            ppd=args.ppd,
            output_map=args.error_map,
            verbosity=args.verbosity
        )
        
        # Output JSON to stdout for easy parsing
        print(json.dumps(result, indent=2))
        
        # Save to file if requested
        if args.output:
            with open(args.output, 'w') as f:
                json.dump(result, f, indent=2)
            if args.verbosity >= 1:
                print(f"\nResults saved to: {args.output}", file=sys.stderr)
        
        # Exit with code based on pass/fail threshold
        # Use recommended threshold of 0.15 for acceptable match
        exit_code = 0 if result["mean"] < 0.15 else 1
        sys.exit(exit_code)
        
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()

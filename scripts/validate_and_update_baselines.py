#!/usr/bin/env python3
"""
Validate and update baseline reference images.

This script handles the scenario where baseline images are missing or questionable.
It cross-validates test outputs against each other and can automatically update
baselines if they pass validation.

Workflow:
1. Check which baselines are missing
2. For missing baselines, validate test output against OTHER backends
3. If test output matches other backends (< threshold), it's good
4. Optionally update the baseline with the validated output

Usage:
    # Validate and show what would be updated
    python3 validate_and_update_baselines.py references/triangle/ test-output/

    # Actually update baselines that pass validation
    python3 validate_and_update_baselines.py references/triangle/ test-output/ --update

    # Custom threshold
    python3 validate_and_update_baselines.py references/triangle/ test-output/ --threshold 0.10
"""

import argparse
import sys
from pathlib import Path
from datetime import datetime
import json

try:
    import flip_evaluator
    import numpy as np
except ImportError as e:
    print(f"Error: Required package not installed: {e}", file=sys.stderr)
    print("Install with: pip install flip-evaluator numpy", file=sys.stderr)
    sys.exit(2)


def compare_images_simple(img1_path, img2_path):
    """Compare two images using FLIP and return mean error."""
    ext = img1_path.suffix.lower()
    dynamic_range = "HDR" if ext == ".exr" else "LDR"
    
    error_map, mean_error, _ = flip_evaluator.evaluate(
        str(img1_path),
        str(img2_path),
        dynamic_range,
        inputsRGB=True,
        applyMagma=True,
        computeMeanError=True,
        parameters={}
    )
    
    return float(mean_error)


def cross_validate_backend(backend_name, test_image, all_test_images, threshold):
    """
    Validate a backend's output by comparing against other backends.
    
    Returns True if the backend matches other backends (mean error < threshold).
    """
    comparisons = []
    
    # Compare against all other backends
    for other_backend, other_test in all_test_images.items():
        if other_backend == backend_name:
            continue
        
        if not other_test.exists():
            continue
        
        mean_error = compare_images_simple(test_image, other_test)
        comparisons.append({
            "other_backend": other_backend,
            "mean_error": mean_error,
            "passed": mean_error < threshold
        })
    
    if not comparisons:
        return False, "No other backends to compare against"
    
    # Check if all comparisons passed
    all_passed = all(c["passed"] for c in comparisons)
    
    if all_passed:
        max_error = max(c["mean_error"] for c in comparisons)
        return True, f"Validated against {len(comparisons)} backend(s), max error: {max_error:.6f}"
    else:
        failed = [c for c in comparisons if not c["passed"]]
        return False, f"Failed validation: {len(failed)} comparison(s) exceeded threshold"


def main():
    parser = argparse.ArgumentParser(
        description="Validate and update baseline reference images"
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
        "--threshold",
        type=float,
        default=0.10,
        help="FLIP error threshold for cross-validation (default: 0.10)"
    )
    parser.add_argument(
        "--update",
        action="store_true",
        help="Actually update baselines (default: dry-run only)"
    )
    parser.add_argument(
        "--backends",
        nargs="+",
        default=["vulkan", "wgpu", "directx"],
        help="Backend names to check (default: vulkan wgpu directx)"
    )
    
    args = parser.parse_args()
    
    if not args.reference_dir.exists():
        print(f"Error: Reference directory not found: {args.reference_dir}", file=sys.stderr)
        sys.exit(2)
    
    if not args.test_dir.exists():
        print(f"Error: Test directory not found: {args.test_dir}", file=sys.stderr)
        sys.exit(2)
    
    print("="*70)
    print("Baseline Validation and Update")
    print("="*70)
    print(f"Reference dir: {args.reference_dir}")
    print(f"Test dir: {args.test_dir}")
    print(f"Threshold: {args.threshold:.2f}")
    print(f"Mode: {'UPDATE' if args.update else 'DRY-RUN'}")
    print("="*70)
    print()
    
    # Find all test images
    all_test_images = {}
    for backend in args.backends:
        test_path = args.test_dir / f"{backend}-triangle.png"
        if test_path.exists():
            all_test_images[backend] = test_path
    
    if len(all_test_images) < 2:
        print(f"Error: Need at least 2 test images for cross-validation", file=sys.stderr)
        print(f"Found: {list(all_test_images.keys())}", file=sys.stderr)
        sys.exit(2)
    
    print(f"Found {len(all_test_images)} test image(s): {', '.join(all_test_images.keys())}")
    print()
    
    # Check each backend
    missing_baselines = []
    valid_for_update = []
    invalid_backends = []
    existing_baselines = []
    
    for backend in args.backends:
        ref_path = args.reference_dir / f"{backend}-triangle.png"
        test_path = all_test_images.get(backend)
        
        if not test_path:
            print(f"⚠️  {backend}: No test image found, skipping")
            continue
        
        if ref_path.exists():
            print(f"✅ {backend}: Baseline exists")
            existing_baselines.append(backend)
            continue
        
        print(f"❌ {backend}: Baseline MISSING")
        missing_baselines.append(backend)
        
        # Cross-validate against other backends
        print(f"   Validating against other backends...")
        is_valid, message = cross_validate_backend(
            backend, test_path, all_test_images, args.threshold
        )
        
        if is_valid:
            print(f"   ✅ {message}")
            valid_for_update.append({
                "backend": backend,
                "test_path": test_path,
                "ref_path": ref_path,
                "message": message
            })
        else:
            print(f"   ❌ {message}")
            invalid_backends.append(backend)
        
        print()
    
    # Summary
    print("="*70)
    print("Summary")
    print("="*70)
    print(f"Existing baselines: {len(existing_baselines)}")
    print(f"Missing baselines: {len(missing_baselines)}")
    print(f"Valid for update: {len(valid_for_update)}")
    print(f"Invalid (not updated): {len(invalid_backends)}")
    print()
    
    if not valid_for_update:
        if missing_baselines:
            print("❌ No baselines can be updated (validation failed)")
            sys.exit(1)
        else:
            print("✅ All baselines exist, nothing to do")
            sys.exit(0)
    
    # Update baselines if requested
    if args.update:
        print("Updating baselines...")
        for item in valid_for_update:
            import shutil
            shutil.copy2(item["test_path"], item["ref_path"])
            print(f"  ✅ Updated {item['backend']}: {item['ref_path']}")
        
        print()
        print("="*70)
        print("✅ Baselines updated successfully!")
        print("="*70)
        print()
        print("Next steps:")
        print("  1. Review changes: git diff references/")
        print("  2. Commit: git add references/")
        print("  3. Commit: git commit -m 'Update validated baselines'")
        print("  4. Push: git push origin main")
    else:
        print("DRY-RUN: Would update the following baselines:")
        for item in valid_for_update:
            print(f"  - {item['backend']}: {item['message']}")
        print()
        print("Run with --update to actually update baselines")
        print()
    
    # Exit with appropriate code
    if invalid_backends:
        print(f"⚠️  Warning: {len(invalid_backends)} backend(s) failed validation")
        sys.exit(1)
    else:
        sys.exit(0)


if __name__ == "__main__":
    main()

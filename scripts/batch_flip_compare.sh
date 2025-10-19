#!/bin/bash
# Batch FLIP comparison script
# Compares multiple test images against reference images

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FLIP_SCRIPT="$SCRIPT_DIR/flip_compare.py"

if [ ! -f "$FLIP_SCRIPT" ]; then
    echo "Error: flip_compare.py not found at $FLIP_SCRIPT"
    exit 1
fi

# Check if arguments provided
if [ $# -lt 2 ]; then
    echo "Usage: $0 <reference_dir> <test_dir> [output_dir]"
    echo ""
    echo "Compares all PNG images in test_dir against reference_dir"
    echo "Output JSON results and error maps to output_dir (default: flip_results)"
    exit 1
fi

REF_DIR="$1"
TEST_DIR="$2"
OUTPUT_DIR="${3:-flip_results}"

# Validate directories
if [ ! -d "$REF_DIR" ]; then
    echo "Error: Reference directory not found: $REF_DIR"
    exit 1
fi

if [ ! -d "$TEST_DIR" ]; then
    echo "Error: Test directory not found: $TEST_DIR"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "Batch FLIP Comparison"
echo "====================="
echo "Reference: $REF_DIR"
echo "Test:      $TEST_DIR"
echo "Output:    $OUTPUT_DIR"
echo ""

# Track results
total=0
passed=0
failed=0

# Compare all PNG files
for test_img in "$TEST_DIR"/*.png; do
    if [ ! -f "$test_img" ]; then
        continue
    fi
    
    filename=$(basename "$test_img")
    ref_img="$REF_DIR/$filename"
    
    if [ ! -f "$ref_img" ]; then
        echo "⚠ Skipping $filename (no reference image)"
        continue
    fi
    
    total=$((total + 1))
    echo "Comparing: $filename"
    
    # Run FLIP comparison
    output_json="$OUTPUT_DIR/${filename%.png}_flip.json"
    error_map="$OUTPUT_DIR/${filename%.png}_error.png"
    
    if python3 "$FLIP_SCRIPT" "$ref_img" "$test_img" \
        --output "$output_json" \
        --error-map "$error_map" \
        --verbosity 1 > /dev/null 2>&1; then
        
        # Extract mean error from JSON
        mean_error=$(python3 -c "import json; print(json.load(open('$output_json'))['mean'])")
        
        if (( $(echo "$mean_error < 0.15" | bc -l) )); then
            echo "  ✓ PASS (mean: $mean_error)"
            passed=$((passed + 1))
        else
            echo "  ✗ FAIL (mean: $mean_error, threshold: 0.15)"
            failed=$((failed + 1))
        fi
    else
        echo "  ✗ ERROR running FLIP"
        failed=$((failed + 1))
    fi
done

echo ""
echo "Results Summary"
echo "==============="
echo "Total:  $total"
echo "Passed: $passed"
echo "Failed: $failed"

if [ $failed -gt 0 ]; then
    exit 1
else
    exit 0
fi

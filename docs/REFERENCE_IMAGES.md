# Reference Image Management

This document describes the reference image system for visual regression testing.

## Overview

Reference images (also called "golden images" or "baseline images") are known-good screenshots that serve as the standard for comparison. When rendering output changes, FLIP compares new screenshots against these references to detect visual regressions.

## Storage Strategy

### Git LFS (Large File Storage)

Reference images are stored using Git LFS to avoid bloating the repository:

**Benefits:**
- Version controlled alongside code
- Doesn't bloat repository size
- Easy to update and track changes
- Works seamlessly with CI/CD

**Setup:**
```bash
# Install Git LFS
# Ubuntu/Debian:
sudo apt install git-lfs

# macOS:
brew install git-lfs

# Windows:
# Download from https://git-lfs.github.com/

# Initialize in repository
git lfs install
git lfs track "references/*.png"
git add .gitattributes
git commit -m "Configure Git LFS for reference images"
```

## Directory Structure

```
rusty_renderer/
├── references/
│   ├── triangle/
│   │   ├── vulkan-triangle.png      # Vulkan reference
│   │   ├── wgpu-triangle.png        # wgpu reference
│   │   └── directx-triangle.png     # DirectX reference
│   ├── cube/
│   │   ├── vulkan-cube.png
│   │   ├── wgpu-cube.png
│   │   └── directx-cube.png
│   └── README.md                     # Reference image metadata
└── .gitattributes                    # Git LFS configuration
```

## Reference Image Metadata

Each reference set should have metadata describing:
- When it was created
- What backend/settings were used
- Image dimensions
- Why it was chosen as reference

Example `references/triangle/README.md`:
```markdown
# Triangle Reference Images

**Created:** 2025-10-19
**Scene:** Basic RGB triangle
**Resolution:** 1280x720
**Backends:** Vulkan, wgpu, DirectX 12

## Vulkan Reference
- **File:** vulkan-triangle.png
- **Backend:** Vulkan 1.3
- **Driver:** Mesa 23.x (lavapipe software renderer)
- **Platform:** Ubuntu 22.04

## wgpu Reference
- **File:** wgpu-triangle.png
- **Backend:** wgpu 0.18
- **Underlying API:** Vulkan
- **Platform:** Ubuntu 22.04

## DirectX Reference
- **File:** directx-triangle.png
- **Backend:** DirectX 12
- **Driver:** WARP software renderer
- **Platform:** Windows 11

## Notes
These images serve as the baseline for visual regression testing.
Minor differences (<0.15 FLIP error) are expected across backends.
```

## Usage

### Comparing Against References

```bash
# Generate current screenshots
cargo run --release -- --backend vulkan --headless \
  --screenshot test-output/vulkan-triangle.png --max-frames 1

# Compare against reference
python3 scripts/flip_compare.py \
  references/triangle/vulkan-triangle.png \
  test-output/vulkan-triangle.png \
  --error-map differences.png

# Or generate comprehensive report
python3 scripts/generate_visual_report.py \
  test-output/ \
  report.html
```

### CI Integration

```yaml
- name: Download reference images
  run: |
    git lfs pull
    cp -r references/triangle screenshots-reference/

- name: Generate test screenshots
  run: |
    # Render with each backend...
    mkdir screenshots-test

- name: Compare against references
  run: |
    python3 scripts/compare_against_baseline.py \
      screenshots-reference/ \
      screenshots-test/ \
      baseline-comparison-report.html
```

## Updating References

### When to Update

Update reference images when:
- **Intentional visual changes** - New features, improved rendering
- **Backend upgrades** - Vulkan/DirectX/wgpu version updates
- **Platform changes** - New OS, driver versions
- **Bug fixes** - Correcting rendering errors

**Don't update for:**
- Random test failures
- Unexplained differences
- Before investigating root cause

### Update Process

1. **Verify Changes Are Intentional**
   ```bash
   # Generate new screenshots
   cargo run --release -- --backend vulkan --headless \
     --screenshot new-vulkan.png --max-frames 1
   
   # Compare visually
   python3 scripts/flip_compare.py \
     references/triangle/vulkan-triangle.png \
     new-vulkan.png \
     --error-map diff.png
   
   # Review diff.png to verify changes are expected
   ```

2. **Update Reference Image**
   ```bash
   # Copy new image to references
   cp new-vulkan.png references/triangle/vulkan-triangle.png
   
   # Update metadata
   nano references/triangle/README.md
   # Note why reference was updated
   ```

3. **Commit Changes**
   ```bash
   git add references/triangle/vulkan-triangle.png
   git add references/triangle/README.md
   git commit -m "Update Vulkan triangle reference

   Reason: Improved anti-aliasing in edge rendering
   FLIP error vs old: 0.23 (significant improvement)
   Reviewed by: [name]
   "
   ```

4. **Update CI Expectations**
   If thresholds need adjustment, update CI configuration.

## Git LFS Configuration

### .gitattributes

```gitattributes
# Reference images use Git LFS
references/**/*.png filter=lfs diff=lfs merge=lfs -text
references/**/*.exr filter=lfs diff=lfs merge=lfs -text

# Also track large test outputs
test-output/**/*.png filter=lfs diff=lfs merge=lfs -text
```

### .gitignore

```gitignore
# Generated test outputs (not references)
screenshots/
flip_results/
test-output/
*.html

# Keep references in version control
!references/
```

## Best Practices

### 1. Minimal Reference Set

Start with minimal references:
- One simple scene per backend
- Gradually add more as needed
- Avoid redundant references

### 2. Document Everything

- Why this reference was chosen
- When it was created
- What hardware/software was used
- Expected FLIP threshold for comparisons

### 3. Review Process

Reference updates should be:
- Reviewed by team member
- Justified in commit message
- Accompanied by comparison data

### 4. Separate Development References

Consider separate references for:
- **Production:** Stable, rarely change
- **Development:** Updated frequently during feature work
- **Platform-specific:** Different OS/driver combinations

### 5. Automation

Automate reference management:
```bash
# Script to update all references
./scripts/update_references.sh --backend vulkan --approve

# Script to validate references
./scripts/validate_references.sh
```

## Reference Image Scripts

### compare_against_baseline.py

Compare test outputs against reference images:

```python
#!/usr/bin/env python3
"""Compare test screenshots against baseline references."""

import argparse
from pathlib import Path
import sys

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("reference_dir", help="Directory with reference images")
    parser.add_argument("test_dir", help="Directory with test screenshots")
    parser.add_argument("output_html", help="Output HTML report")
    parser.add_argument("--threshold", type=float, default=0.15,
                       help="FLIP error threshold (default: 0.15)")
    
    args = parser.parse_args()
    
    # Compare each test image against its reference
    # Generate report with pass/fail status
    # Exit with error if any comparison exceeds threshold

if __name__ == "__main__":
    main()
```

### update_references.sh

Update reference images from test outputs:

```bash
#!/bin/bash
# Update reference images after review

set -e

BACKEND=${1:-all}
APPROVE=${2:-false}

if [ "$APPROVE" != "--approve" ]; then
    echo "This will update reference images."
    echo "Have you reviewed the changes?"
    echo "Run with --approve to confirm."
    exit 1
fi

# Copy test outputs to references
cp test-output/vulkan-triangle.png references/triangle/
cp test-output/wgpu-triangle.png references/triangle/
cp test-output/directx-triangle.png references/triangle/

# Update metadata
echo "Updated: $(date)" >> references/triangle/README.md

# Commit
git add references/
git commit -m "Update reference images for $BACKEND"

echo "✅ References updated"
```

## CI Workflow

### With References

```yaml
visual-regression-with-baseline:
  name: Visual Regression (Against Baseline)
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        lfs: true  # Download LFS files
    
    - name: Setup Python & FLIP
      run: |
        pip install flip-evaluator numpy pillow
    
    - name: Generate test screenshots
      run: |
        mkdir test-screenshots
        cargo run --release -- --backend vulkan --headless \
          --screenshot test-screenshots/vulkan-triangle.png
    
    - name: Compare against baseline
      run: |
        python3 scripts/compare_against_baseline.py \
          references/triangle/ \
          test-screenshots/ \
          baseline-report.html \
          --threshold 0.10
    
    - name: Upload baseline comparison
      if: failure()
      uses: actions/upload-artifact@v4
      with:
        name: baseline-comparison-failed
        path: |
          baseline-report.html
          test-screenshots/
```

## Troubleshooting

### Git LFS Not Working

```bash
# Check LFS status
git lfs status

# Pull LFS files
git lfs pull

# Verify files are tracked
git lfs ls-files
```

### Large Repository Size

```bash
# LFS files shouldn't be in regular git
# Verify .gitattributes is correct
cat .gitattributes

# Migrate existing large files to LFS
git lfs migrate import --include="*.png" --everything
```

### Reference Drift

If references slowly drift over time:
- Review hardware changes
- Check driver/OS updates
- Consider platform-specific references
- Tighten FLIP thresholds

## Future Enhancements

Planned improvements:

1. **Automatic Reference Updates**
   - Bot PR when references need updating
   - Automated approval for minor changes

2. **Multi-Platform References**
   - Separate references per OS
   - Per-GPU-vendor references

3. **Historical Tracking**
   - Track FLIP error trends over time
   - Alert on gradual drift

4. **Smart Threshold Adjustment**
   - Learn acceptable ranges
   - Auto-adjust thresholds per comparison

## Status

🚧 **Planned** - Implementation upcoming

This system will be implemented after:
- ✅ FLIP integration complete
- ✅ HTML report generation working
- ✅ All backends in CI
- 🎯 Reference image storage (next step)

---

**See Also:**
- [FLIP Integration Guide](FLIP_INTEGRATION.md)
- [M5 Planning](M5_PLANNING.md)
- [Visual Testing README](../src/testing/README.md)

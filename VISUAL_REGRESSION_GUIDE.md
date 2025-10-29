# Visual Regression Testing Guide

## Overview

Our CI includes comprehensive visual regression testing to catch unintended rendering changes.

## How It Works

### 1. Golden References
Located in `references/gltf_textured/`:
- `gltf_textured_vulkan.png` - Known good Vulkan output
- `gltf_textured_directx.png` - Known good DirectX output

These are the baseline "correct" images that CI compares against.

### 2. Three Types of Comparisons

#### A. Backend Parity (Vulkan vs DirectX)
- **Purpose**: Ensure both backends render similarly
- **Comparison**: Current Vulkan output vs Current DirectX output
- **Tolerance**: ~14% RMSE expected (coordinate system differences)
- **Status**: Warning only (not a failure)

#### B. Vulkan Regression Check
- **Purpose**: Detect unintended Vulkan rendering changes
- **Comparison**: Current Vulkan output vs Golden Vulkan reference
- **Tolerance**: < 1% RMSE (strict)
- **Status**: Warning if differs

#### C. DirectX Regression Check
- **Purpose**: Detect unintended DirectX rendering changes
- **Comparison**: Current DirectX output vs Golden DirectX reference
- **Tolerance**: < 1% RMSE (strict)
- **Status**: Warning if differs

### 3. Visual Reports

CI generates an HTML report with:
- Side-by-side comparisons
- Difference heatmaps
- FLIP perceptual metrics
- Embedded images for easy review

**Access**: Download `visual-regression-results` artifact from CI run

## Updating Golden References

### When to Update

✅ **Update when**:
- You intentionally changed rendering
- Fixed a rendering bug
- Improved visual quality
- Added new features that change output

❌ **Don't update when**:
- You see artifacts or corruption
- Output looks wrong
- Change was unintentional
- You're not sure what changed

### How to Update

#### Option 1: Automated Script (Recommended)
```bash
# Update Vulkan reference (on Linux)
./scripts/update_golden_references.sh --vulkan

# Verify it looks correct
open references/gltf_textured/gltf_textured_vulkan.png

# Commit if good
git add references/gltf_textured/
git commit -m "Update Vulkan golden reference - [reason]"
```

#### Option 2: From CI Artifacts
1. Find a successful CI run with correct rendering
2. Download `screenshots-vulkan` or `screenshots-directx` artifact
3. Extract and visually verify the screenshot
4. Copy to `references/gltf_textured/gltf_textured_[backend].png`
5. Commit with descriptive message

#### Option 3: Manual Render
```bash
# Render with Vulkan
cargo run --release -- \
    --scene scenes/gltf_textured.toml \
    --backend vulkan \
    --pipeline forward \
    --headless \
    --screenshot references/gltf_textured/gltf_textured_vulkan.png

# Verify and commit
```

## Investigating Failures

### Step 1: Download CI Artifacts
```bash
gh run download <run-id> --name visual-regression-results
```

### Step 2: Open HTML Report
```bash
open visual-regression-report.html
```

The report shows:
- What changed
- How much it changed (RMSE, FLIP scores)
- Visual diff highlighting differences
- Side-by-side comparison

### Step 3: Analyze the Difference

**Small differences (< 1% RMSE)**:
- Likely acceptable
- May be floating-point precision
- Verify visually

**Medium differences (1-5% RMSE)**:
- Review carefully
- Could be intentional or a bug
- Check recent changes

**Large differences (> 5% RMSE)**:
- Likely significant change
- Investigate code changes
- May need to update reference

### Step 4: Take Action

**If change is good**:
- Update golden reference
- Document why in commit message

**If change is bad**:
- Fix the rendering issue
- Don't update reference

**If unsure**:
- Ask for review
- Compare with previous good output
- Test locally

## Texture Artifacts Warning

If you see random noise/artifacts in screenshots:
1. **DO NOT** use them as golden references
2. Check texture loading
3. Verify shader compilation
4. Test rendering locally
5. Only update references when output is clean

Common causes:
- Uninitialized texture data
- Missing texture files
- Incorrect texture format
- Shader errors

## CI Configuration

Visual regression testing is in `.github/workflows/ci.yml`:

```yaml
- name: Compare against golden references
  run: |
    # Vulkan check
    python3 scripts/flip_compare.py \
      references/gltf_textured/gltf_textured_vulkan.png \
      screenshots/vulkan/gltf_textured.png \
      --error-map comparisons/golden/vulkan_regression.png
      
    # DirectX check
    python3 scripts/flip_compare.py \
      references/gltf_textured/gltf_textured_directx.png \
      screenshots/directx/gltf_textured.png \
      --error-map comparisons/golden/directx_regression.png
```

## Best Practices

1. **Always verify visually** before committing new references
2. **Document changes** in commit messages
3. **Keep references up-to-date** with intentional changes
4. **Don't ignore warnings** - investigate what changed
5. **Use descriptive commit messages** when updating references

## Troubleshooting

### "No golden reference" warning
- References don't exist for that scene
- Create them with update script

### False positives
- Check tolerance in `flip_compare.py`
- Minor differences are expected
- Compare with recent good output

### Backend parity warnings
- Normal to see ~14% difference
- Different coordinate systems
- Will improve as we refine rendering

---

*For more details, see `references/gltf_textured/README.md`*

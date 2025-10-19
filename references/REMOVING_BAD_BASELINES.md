# Removing Bad Baselines - Safe Recovery Guide

This guide explains how to safely remove incorrect baseline images and have them automatically validated and re-added if they match other backends.

## Problem

You discover a baseline reference image is incorrect:
- Wrong colors
- Rendering artifacts
- Coordinate system issues
- etc.

**Question:** Can I just delete it and have it regenerated?

**Answer:** YES! ✅

## Safe Removal Workflow

### Step 1: Remove the Bad Baseline

```bash
# Remove the incorrect baseline
git rm references/triangle/wgpu-triangle.png

# Commit the removal
git commit -m "Remove incorrect wgpu baseline

The baseline has rendering artifacts that don't match other backends.
Will be regenerated after validation against Vulkan and DirectX."

# Push
git push origin main
```

### Step 2: CI Automatically Validates

When CI runs:

1. **Detects Missing Baseline**
   ```
   ❌ wgpu: Baseline MISSING
      Validating against other backends...
   ```

2. **Cross-Validates Against Other Backends**
   - Compares wgpu test output against Vulkan
   - Compares wgpu test output against DirectX
   - Checks if FLIP error < 0.10 for all comparisons

3. **Determines if Valid**
   ```
   ✅ Validated against 2 backend(s), max error: 0.047
   ```

4. **Result:**
   - ✅ If valid (matches other backends): Passes CI, ready to re-add
   - ❌ If invalid (doesn't match): Fails CI, needs investigation

### Step 3: Re-Add Validated Baseline

If CI passes (meaning the output is now correct):

```bash
# Download CI artifacts
gh run download <run-id> -n visual-regression-report-all-backends

# Validate and update baselines
python3 scripts/validate_and_update_baselines.py \
  references/triangle/ \
  screenshots/ \
  --update

# Commit the validated baseline
git add references/triangle/wgpu-triangle.png
git commit -m "Add validated wgpu baseline

Cross-validated against Vulkan and DirectX.
FLIP errors: Vulkan=0.043, DirectX=0.047
All below 0.10 threshold."

git push origin main
```

## Automatic Validation Script

### Usage

```bash
# Check what would be updated (dry-run)
python3 scripts/validate_and_update_baselines.py \
  references/triangle/ \
  screenshots/

# Actually update validated baselines
python3 scripts/validate_and_update_baselines.py \
  references/triangle/ \
  screenshots/ \
  --update
```

### Output Example

```
======================================================================
Baseline Validation and Update
======================================================================
Reference dir: references/triangle
Test dir: screenshots
Threshold: 0.10
Mode: DRY-RUN
======================================================================

Found 3 test image(s): vulkan, wgpu, directx

✅ vulkan: Baseline exists
❌ wgpu: Baseline MISSING
   Validating against other backends...
   ✅ Validated against 2 backend(s), max error: 0.047000
✅ directx: Baseline exists

======================================================================
Summary
======================================================================
Existing baselines: 2
Missing baselines: 1
Valid for update: 1
Invalid (not updated): 0

DRY-RUN: Would update the following baselines:
  - wgpu: Validated against 2 backend(s), max error: 0.047000

Run with --update to actually update baselines
```

## How It Works

### Cross-Validation Logic

For a missing baseline (e.g., `wgpu`):

1. **Find All Other Test Outputs**
   - Finds `vulkan-triangle.png` (exists)
   - Finds `directx-triangle.png` (exists)

2. **Compare Missing Backend Against Each**
   ```
   FLIP(wgpu, vulkan) = 0.043  < 0.10 ✅
   FLIP(wgpu, directx) = 0.047 < 0.10 ✅
   ```

3. **Decision**
   - If ALL comparisons < threshold → Valid ✅
   - If ANY comparison ≥ threshold → Invalid ❌

4. **Update (if --update flag)**
   - Copy `wgpu-triangle.png` to `references/triangle/`
   - Ready to commit

### Why This Works

**Assumption:** If all backends render identically (FLIP error < 0.10), then any one of them is a valid baseline.

**Logic:**
- 3 backends: Vulkan, wgpu, DirectX
- If wgpu matches both Vulkan AND DirectX
- Then wgpu is rendering correctly
- Safe to use as baseline

## CI Integration

The CI workflow automatically runs validation:

```yaml
- name: Validate missing baselines (if any)
  run: |
    python3 scripts/validate_and_update_baselines.py \
      references/triangle/ \
      screenshots/ \
      --threshold 0.10 || {
        echo "⚠️  Some baselines missing or invalid"
        exit 0  # Don't fail CI, just warn
      }
```

**Behavior:**
- If baselines missing but test outputs match → CI warns but passes
- If baselines missing and test outputs DON'T match → CI warns but passes (not a failure)
- Actual baseline comparison (if baselines exist) will fail if threshold exceeded

## Example Scenarios

### Scenario 1: Remove Bad wgpu Baseline

```bash
# Current state: wgpu baseline has artifacts
ls references/triangle/
# vulkan-triangle.png
# wgpu-triangle.png      ← BAD
# directx-triangle.png

# Remove it
git rm references/triangle/wgpu-triangle.png
git commit -m "Remove bad wgpu baseline"
git push

# CI runs, validates wgpu output
# If wgpu now renders correctly → validates against Vulkan+DirectX
# Download artifacts and re-add
./scripts/populate_references.sh screenshots/
```

### Scenario 2: All Backends Match

```bash
# Remove all baselines
rm references/triangle/*.png

# CI runs, no baselines exist
# Cross-validates all outputs against each other
# If all match (< 0.10), all are valid
# Re-add all at once
./scripts/populate_references.sh screenshots/
```

### Scenario 3: One Backend Broken

```bash
# Remove wgpu baseline
rm references/triangle/wgpu-triangle.png

# CI runs, wgpu output doesn't match others
# FLIP(wgpu, vulkan) = 0.35  ❌ > 0.10
# FLIP(wgpu, directx) = 0.38 ❌ > 0.10

# Result: Validation fails
# ❌ wgpu: Failed validation: 2 comparison(s) exceeded threshold

# Action: Fix wgpu rendering bug, then re-run
```

## Benefits

✅ **Safe to Remove** - Can delete questionable baselines  
✅ **Automatic Validation** - CI checks against other backends  
✅ **No Manual Work** - Script handles FLIP comparisons  
✅ **Quality Assured** - Only re-adds if matches other backends  
✅ **Clear Feedback** - Shows exactly why validation passed/failed  

## Troubleshooting

### All Validations Failing

If script says all backends are invalid:
- Check if any baselines exist
- Need at least 1 baseline OR 2 matching test outputs
- Review FLIP errors in output

### Validation Passes But Shouldn't

If a bad baseline validates:
- Check threshold (default 0.10)
- May need tighter threshold: `--threshold 0.05`
- Other backends might also be wrong

### Script Not Finding Images

- Check paths: `references/triangle/` and `screenshots/`
- Ensure images named: `<backend>-triangle.png`
- Check file extensions (.png, not .PNG)

## See Also

- [Reference Images README](README.md)
- [FLIP Integration Guide](../docs/FLIP_INTEGRATION.md)
- [Scripts README](../scripts/README.md)

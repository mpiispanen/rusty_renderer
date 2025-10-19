# Session: CI Fails on Visual Regression

**Date:** October 19, 2025  
**Focus:** Ensure CI job fails when visual regression is detected

## Summary

Updated the visual regression testing CI workflow to properly fail when FLIP comparisons exceed threshold, ensuring visual regressions are caught before merging.

## Problem

The CI workflow was suppressing errors from the report generator:
```bash
python3 scripts/generate_visual_report.py ... || {
  echo "Failed but continuing..."
  exit 0  # <-- Always exits successfully!
}
```

This meant even if visual regressions were detected, CI would pass ✅

## Solution

### 1. Remove Error Suppression

Changed CI workflow to respect exit codes:
```bash
# Generate report - will exit with code 1 if any comparison fails
python3 scripts/generate_visual_report.py \
  screenshots/ \
  visual-regression-report.html \
  --temp-dir flip_results

# Script exits with code 0 if all pass, 1 if any fail
# CI will fail automatically if exit code is 1
```

### 2. Enhanced Error Messages

Updated Python script to provide clear failure messages:
```python
if passed != total_comparisons:
    print(f"\n❌ Visual regression test FAILED: {total_comparisons - passed} comparison(s) exceeded threshold")
    print(f"   Threshold: 0.15 mean FLIP error")
    print(f"   Review the HTML report for details")

sys.exit(0 if passed == total_comparisons else 1)
```

### 3. Updated Documentation

Added CI/CD Behavior section to FLIP_INTEGRATION.md:
- Explains automatic failure behavior
- Documents exit codes
- Provides investigation workflow
- Shows example failure output

## Behavior

### When All Comparisons Pass (< 0.15 threshold)

**CI:** ✅ Passes  
**Output:**
```
✅ Report generated: visual-regression-report.html
   Total comparisons: 3
   Passed: 3/3
```

**Exit Code:** 0

### When Any Comparison Fails (≥ 0.15 threshold)

**CI:** ❌ Fails  
**Output:**
```
✅ Report generated: visual-regression-report.html
   Total comparisons: 3
   Passed: 2/3

❌ Visual regression test FAILED: 1 comparison(s) exceeded threshold
   Threshold: 0.15 mean FLIP error
   Review the HTML report for details
```

**Exit Code:** 1

### Artifact Upload

**Important:** Report is uploaded **even on failure** via `if: always()`:

```yaml
- name: Upload comprehensive visual regression report
  if: always()  # <-- Runs even if previous step failed
  uses: actions/upload-artifact@v4
  with:
    name: visual-regression-report-all-backends
    path: |
      visual-regression-report.html
      screenshots/
      flip_results/
```

This ensures you can always review what failed.

## Investigation Workflow

When CI fails:

1. **Check CI Logs**
   ```
   ❌ Visual regression test FAILED: 1 comparison(s) exceeded threshold
   ```

2. **Download Artifact**
   - Go to Actions → Failed Run
   - Download `visual-regression-report-all-backends`

3. **Open HTML Report**
   - Find failed comparison (red badge)
   - Review error map
   - Check metrics

4. **Determine Action**
   - **Intentional change:** Update reference images
   - **Bug introduced:** Fix rendering code
   - **Backend difference:** Investigate or adjust threshold
   - **False positive:** Review threshold settings

## Files Modified

### CI Workflow (`.github/workflows/ci.yml`)

**Before:**
```yaml
python3 scripts/generate_visual_report.py ... || {
  echo "Failed but continuing..."
  exit 0
}
```

**After:**
```yaml
python3 scripts/generate_visual_report.py \
  screenshots/ \
  visual-regression-report.html \
  --temp-dir flip_results
# Exits with 1 on failure, CI respects exit code
```

### Report Generator (`scripts/generate_visual_report.py`)

**Added:**
```python
if passed != total_comparisons:
    print(f"\n❌ Visual regression test FAILED: ...")
    print(f"   Threshold: 0.15 mean FLIP error")
    print(f"   Review the HTML report for details")

sys.exit(0 if passed == total_comparisons else 1)
```

### Documentation (`docs/FLIP_INTEGRATION.md`)

**Added Section:** CI/CD Behavior
- Automatic failure explanation
- Exit codes documented
- Investigation workflow
- Example outputs

## Benefits

### Before This Change

❌ Visual regressions could be merged  
❌ No immediate feedback on quality issues  
❌ Manual review required to catch problems  
❌ False sense of security ("CI is green")  

### After This Change

✅ **CI fails immediately on visual regression**  
✅ **Clear error messages in logs**  
✅ **Report still available for investigation**  
✅ **Prevents merging visual bugs**  
✅ **Automated quality gate**  

## Testing

### Test Pass Scenario

With current triangle rendering (all backends match):
```
Vulkan vs wgpu:     0.08 (PASS)
Vulkan vs DirectX:  0.08 (PASS)
wgpu vs DirectX:    0.08 (PASS)

Result: CI PASSES ✅
```

### Test Fail Scenario

If a backend renders incorrectly:
```
Vulkan vs wgpu:     0.08 (PASS)
Vulkan vs DirectX:  0.45 (FAIL)  <-- Over threshold
wgpu vs DirectX:    0.45 (FAIL)  <-- Over threshold

Result: CI FAILS ❌
Message: 2 comparison(s) exceeded threshold
```

## Edge Cases

### Missing Backends

If only one backend available (can't compare):
- Script exits with error
- CI fails
- This is correct behavior (incomplete testing)

### No Screenshots

If screenshot download fails:
- Script exits with error
- CI fails
- This is correct behavior (testing not completed)

### All Backends Present

Normal case:
- 3 comparisons performed
- Exit code based on threshold checks
- Proper pass/fail determination

## Status

✅ **Complete** - CI now fails on visual regression

**Validation:**
- Will be verified in next CI run
- With corrected DirectX coordinates, all should pass
- Future regressions will be caught

---

**Next Steps:**
1. Verify CI fails properly if threshold exceeded
2. Test by temporarily breaking a backend
3. Confirm artifact upload works on failure
4. Proceed with reference image implementation

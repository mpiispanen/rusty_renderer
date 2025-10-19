# Reference Images

This directory contains baseline (reference) images for visual regression testing. These images represent known-good rendering output that new builds are compared against.

## Purpose

Reference images serve as the "golden standard" for visual correctness. When code changes are made, the renderer generates new screenshots which are compared against these references using FLIP (perceptual image comparison). If differences exceed acceptable thresholds, the build fails, alerting developers to potential visual regressions.

## Structure

```
references/
├── triangle/          # Basic triangle test scene
│   ├── vulkan-triangle.png
│   ├── wgpu-triangle.png
│   └── directx-triangle.png
└── README.md
```

Each subdirectory contains reference images for a specific test scene, with one image per backend.

## Storage with Git LFS

Reference images are stored using Git LFS (Large File Storage) to avoid bloating the repository:

- PNG files are tracked by LFS (see `.gitattributes`)
- Images are versioned alongside code
- CI automatically pulls LFS files
- Updates are tracked in git history

## When to Update References

Update reference images when:

1. **Intentional visual changes** - New features or rendering improvements
2. **Bug fixes** - Correcting rendering errors that were in the baseline
3. **Backend updates** - Vulkan/DirectX/wgpu version changes that affect output
4. **Platform changes** - New OS or driver versions with different rasterization

**DO NOT update for:**
- Unexplained differences
- Random test failures
- Before investigating root cause

## Update Process

### Standard Update (Known Good Images)

1. **Generate new screenshots** with the updated renderer
2. **Compare against current references** using FLIP
3. **Review differences** - ensure they're intentional
4. **Copy new images** to `references/` directory
5. **Update metadata** in scene-specific README
6. **Commit changes** with explanation

### Remove Bad Baseline (Safe Recovery)

If you discover a baseline is incorrect:

1. **Remove the bad baseline**
   ```bash
   git rm references/triangle/wgpu-triangle.png
   git commit -m "Remove incorrect wgpu baseline"
   ```

2. **CI will validate replacement**
   - CI detects missing baseline
   - Cross-validates test output against other backends
   - If test matches other backends (< 0.10 error), it's valid

3. **Update with validated baseline**
   ```bash
   # After CI passes, download artifacts
   ./scripts/populate_references.sh screenshots/
   
   # Or manually validate and update
   python3 scripts/validate_and_update_baselines.py \
     references/triangle/ \
     screenshots/ \
     --update
   ```

**Benefits:**
- Safe to remove questionable baselines
- Automatic cross-validation
- No manual FLIP comparison needed
- Quality assured by comparison to other backends

Example:
```bash
# Generate new screenshots
cargo run --release -- --backend vulkan --headless \
  --screenshot test-output/vulkan-triangle.png

# Compare against reference
python3 scripts/flip_compare.py \
  references/triangle/vulkan-triangle.png \
  test-output/vulkan-triangle.png \
  --error-map diff.png

# Review diff.png - if acceptable, update reference
cp test-output/vulkan-triangle.png references/triangle/

# Commit
git add references/triangle/vulkan-triangle.png
git commit -m "Update Vulkan triangle reference: improved AA"
```

## Current Baselines

### Triangle Scene

**Created:** 2025-10-19  
**Resolution:** 1280x720  
**Scene Description:** Simple RGB triangle - Red at bottom, Green at top-right, Blue at top-left

| Backend | Image | Notes |
|---------|-------|-------|
| Vulkan | vulkan-triangle.png | Using lavapipe (Mesa software renderer) |
| wgpu | wgpu-triangle.png | Using Vulkan backend on Linux |
| DirectX | directx-triangle.png | Using WARP (software renderer) on Windows |

**Expected FLIP Errors:**
- Vulkan vs wgpu: < 0.05 (should be nearly identical)
- Vulkan vs DirectX: < 0.05 (after coordinate fixes)
- wgpu vs DirectX: < 0.05 (after coordinate fixes)

## CI Integration

The CI workflow:

1. **Checkout with LFS** - Downloads reference images
2. **Generate test screenshots** - Renders with each backend
3. **Compare against baselines** - Uses FLIP for comparison
4. **Generate report** - Creates HTML report with results
5. **Fail if threshold exceeded** - CI fails if mean error ≥ 0.10

See `.github/workflows/ci.yml` for implementation details.

## Troubleshooting

### LFS not pulling files

```bash
# Manually pull LFS files
git lfs pull

# Verify LFS is working
git lfs ls-files
```

### Large repository size

If the repository is getting large despite LFS:

```bash
# Check what's taking space
git lfs ls-files -s

# Verify .gitattributes is correct
cat .gitattributes
```

### Reference drift over time

If references slowly drift despite no intentional changes:
- Check for OS/driver updates
- Review hardware changes
- Consider platform-specific references
- Tighten FLIP thresholds

## Best Practices

1. **Keep references minimal** - Only add references for critical test cases
2. **Document changes** - Always explain why references were updated
3. **Review carefully** - Visual changes should be intentional
4. **Test locally first** - Verify changes before pushing
5. **Consider separate dev/prod** - Use different baselines for development vs release

## See Also

- [FLIP Integration Guide](../docs/FLIP_INTEGRATION.md)
- [Reference Images Management](../docs/REFERENCE_IMAGES.md)
- [Visual Testing README](../src/testing/README.md)

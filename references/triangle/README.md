# Triangle Scene Reference Images

**Created:** 2025-10-19  
**Scene:** Basic RGB colored triangle  
**Resolution:** 1280x720  
**Purpose:** Baseline test for visual regression

## Description

Simple triangle with vertex colors:
- Bottom center: Red
- Top right: Green
- Top left: Blue

This is the fundamental test case for ensuring consistent rendering across all backends.

## Reference Images

### Vulkan (vulkan-triangle.png)
- **Backend:** Vulkan 1.3
- **Driver:** Mesa lavapipe (software renderer)
- **Platform:** Ubuntu 24.04 (GitHub Actions)
- **Created:** 2025-10-19
- **Notes:** Reference backend - all others are compared to this

### wgpu (wgpu-triangle.png)
- **Backend:** wgpu 0.18
- **Underlying API:** Vulkan
- **Platform:** Ubuntu 24.04 (GitHub Actions)
- **Created:** 2025-10-19
- **Notes:** Minor differences due to shader pipeline

### DirectX 12 (directx-triangle.png)
- **Backend:** DirectX 12
- **Driver:** WARP (software renderer)
- **Platform:** Windows Server 2022 (GitHub Actions)
- **Created:** 2025-10-19
- **Notes:** Y-axis flipped in shader to match Vulkan output

## Expected FLIP Errors

When comparing against these baselines:

| Comparison | Expected Mean Error | Status |
|-----------|---------------------|--------|
| Vulkan (current vs baseline) | < 0.03 | EXCELLENT |
| wgpu (current vs baseline) | < 0.05 | EXCELLENT/GOOD |
| DirectX (current vs baseline) | < 0.03 | EXCELLENT |

**Threshold:** 0.10 mean FLIP error (CI fails if exceeded)

## Update History

- **2025-10-19:** Initial baseline images created from CI run

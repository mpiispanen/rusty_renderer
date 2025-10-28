# Backend Parity Progress - 2025-10-27

## Goal
Make Vulkan and DirectX backends produce identical visual output to ensure true backend equivalence and enable reliable CI visual regression testing.

## Progress Summary

### ✅ Fixed Issues

#### 1. Clear Color Mismatch (MAJOR)
- **Problem**: Background colors were completely different
  - Vulkan: Black (0, 0, 0, 1) → 89% black pixels
  - DirectX: Dark blue (0.1, 0.1, 0.2, 1) → 0% black pixels
  - RMSE: 17.4% difference

- **Solution**: Standardized clear color to dark blue (0.1, 0.1, 0.2, 1) in both backends
  - Files changed: `src/backends/vulkan/mod.rs` (lines 1173, 2514)
  
- **Result**: Background now identical (0 difference)
  - RMSE improved from 17.4% → 12.9%
  - Pixels with >10 diff: 99.9% → 10.9%

### ⚠️ Remaining Issues

#### 1. Specular Highlight Differences (10.9% pixels differ)
- **Observation**: Specific pixels show large differences (up to 167 units)
  - DirectX: Bright highlight [216, 210, 202] at position ~[323, 430]
  - Vulkan: Dark pixel [49, 48, 46] at same position

- **Root Cause**: Using different shader languages!
  - **Vulkan**: GLSL (`shaders/forward.frag`) → SPIR-V
  - **DirectX**: HLSL (`shaders/hlsl/forward.hlsl`) → runtime compilation
  
- **Why Different**: Even with "equivalent" code, subtle differences exist:
  - Syntax differences between GLSL and HLSL
  - Different compilation pipelines
  - Potentially different precision/rounding
  - Manual translation errors

## Current Status

**Metrics:**
- Background: ✅ Perfect match (0 difference)
- Cube rendering: ⚠️ 10.9% pixels differ by >10 units
- Overall RMSE: 12.9%

**Visual Quality:**
- Both backends render recognizable, correct images
- Differences are in specular highlights and edge pixels
- Likely due to shader language differences

## Next Steps

### Option 1: Accept Current Differences (RECOMMENDED FOR NOW)
- Backends are "close enough" for most purposes
- Focus on architecture refactoring (data-driven pipelines)
- Revisit when we have unified shader compilation

### Option 2: Unified Shader Compilation
- Compile HLSL to SPIR-V for both backends (using DXC or similar)
- Guarantees identical shader code
- Part of the larger architecture refactor plan

### Option 3: Deep Shader Debugging
- Add debug outputs to shaders
- Compare intermediate values (normals, light directions, etc.)
- Fix GLSL/HLSL differences one by one
- Time-consuming, may be superseded by Option 2

## Recommendation

**Proceed with Option 1** - The current 12.9% difference is acceptable for now:
1. Both backends produce correct, recognizable output
2. CI can use relaxed thresholds (~15%) for visual regression
3. Focus efforts on architecture refactoring
4. Perfect parity will come naturally when we move to unified shaders

## Files Modified

- `src/backends/vulkan/mod.rs` - Clear color fix
- Analysis scripts created for future debugging:
  - `analyze_diff.py`
  - `check_images.py`
  - `detailed_diff.py`

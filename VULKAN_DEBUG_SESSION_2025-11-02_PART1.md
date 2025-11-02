# Vulkan Rendering Debug Session - November 2, 2025

## UPDATE: ROOT CAUSE IDENTIFIED!

**Descriptor set layout mismatch between hardcoded layout and HLSL shader.**

## Issue
Vulkan backend only rendering clear color, no geometry visible.

## Root Cause: Descriptor Set Layout Mismatch

### Validation Errors (with --debug flag)

```
[ERROR] vkCreateGraphicsPipelines(): pCreate Infos[0].pStages[1] SPIR-V uses descriptor 
[Set 0, Binding 0, variable "diffuseTexture"] of type VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER 
but expected VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER

[ERROR] vkCmdDraw(): descriptor [Set 0, Binding 2, "PushConstants"] has never been 
updated via vkUpdateDescriptorSets()
```

### The Problem

**Hardcoded descriptor layout** (`create_uniform_descriptor_layouts()`):
- Binding 0: Camera (UNIFORM_BUFFER) ✓
- Binding 1: Lighting (UNIFORM_BUFFER) ✓  
- Binding 2: Diffuse texture (COMBINED_IMAGE_SAMPLER) ✗
- Binding 3: Material (UNIFORM_BUFFER) ✗

**HLSL shader** (`forward_simple.hlsl`):
- `register(b0)`: Camera uniforms
- `register(b1)`: Lighting uniforms
- `register(b2)`: PushConstants ← **DXC compiles this as descriptor binding!**

**DXC Issue**: When compiling HLSL to SPIR-V, `cbuffer PushConstants : register(b2)` is being treated as a uniform buffer descriptor instead of Vulkan push constants.

### Solution

Fix the render graph pipeline compilation to create proper descriptor layouts based on the pipeline description, not use the hardcoded one. The hardcoded `create_pipeline()` and descriptor layouts should be removed.

### Status

**FIXED:**
1. ✅ Updated HLSL shader to use `[[vk::push_constant]]` attribute for push constants
2. ✅ Removed hardcoded pipeline creation calls from init functions
3. ✅ Implemented per-pass pipeline layouts and descriptor layouts  
4. ✅ Shader now compiles correctly with DXC (no more texture/sampler bindings)

**CURRENT ISSUE:**
Application crashes after graph compilation when running in windowed mode. Need to investigate crash cause with backtrace.

**FILES MODIFIED:**
- `shaders/hlsl/forward_simple.hlsl` - Fixed push constants syntax for Vulkan
- `src/backends/vulkan/mod.rs` - Added pipeline layout caching per pass, removed hardcoded pipeline creation

---

## Previous Investigation


### 1. Confirmed rendering pipeline is executing
- Logs show draw calls being issued (36 vertices per frame)
- Command buffers being submitted
- No validation errors during rendering

### 2. Identified shader/uniform mismatches

#### Problem 1: Material buffer reference
- **Shader**: Expected `MaterialUniforms` at binding 3 (b3)
- **App**: Not binding any material buffer
- **Fix**: Removed material buffer reference from shader, using only vertex colors

#### Problem 2: Push constants stage mismatch
- **Shader**: Push constants only used in vertex stage
- **App**: Was pushing to VERTEX | FRAGMENT stages (0x11)
- **Fix**: Changed to VERTEX stage only (0x1)

#### Problem 3: Camera uniforms structure mismatch
- **Shader expects**: 
  ```hlsl
  cbuffer CameraUniforms : register(b0) {
      float4x4 viewProj;  // Single pre-multiplied matrix (64 bytes)
  }
  ```
- **App was sending**:
  ```rust
  struct CameraUniforms {
      view: [[f32; 4]; 4],      // 64 bytes
      proj: [[f32; 4]; 4],      // 64 bytes  
      view_pos: [f32; 3],       // 12 bytes
      _padding: f32,            // 4 bytes
  }  // Total: 144 bytes
  ```
- **Fix**: Changed to send pre-multiplied viewProj matrix (64 bytes total)

#### Problem 4: Lighting uniforms structure mismatch
- **Shader expects**:
  ```hlsl
  struct Light {
      uint lightType;           // 4 bytes
      uint padding1;            // 4 bytes
      uint padding2;            // 4 bytes
      uint padding3;            // 4 bytes
      float4 positionOrDirection;  // 16 bytes
      float4 colorIntensity;    // 16 bytes
  };  // Total: 48 bytes per light
  
  cbuffer LightingUniforms : register(b1) {
      float4 ambientLightCount;  // 16 bytes
      Light lights[MAX_LIGHTS];  // 48 * 8 = 384 bytes
  }  // Total: 400 bytes
  ```
- **App was sending**:
  ```rust
  struct LightingUniforms {
      ambient: [f32; 3],        // 12 bytes
      _padding1: f32,           // 4 bytes
      light_dir: [f32; 3],      // 12 bytes
      _padding2: f32,           // 4 bytes
      light_color: [f32; 3],    // 12 bytes
      light_intensity: f32,     // 4 bytes
  }  // Total: 48 bytes
  ```
- **Fix**: Changed to match shader structure with full lights array (400 bytes total)

## Changes Made

### Files Modified
1. **shaders/hlsl/forward_simple.hlsl**
   - Removed MaterialUniforms cbuffer
   - Changed fragment shader to use only vertex colors

2. **src/passes/forward_simple.rs**
   - Fixed push constants stage flags (VERTEX only)
   - Updated camera buffer size: 144 bytes → 64 bytes  
   - Updated lighting buffer size: 48 bytes → 400 bytes

3. **src/app.rs**
   - Added `mul_mat4()` helper function
   - Changed CameraUniforms to single viewProj matrix
   - Changed LightingUniforms to match shader structure with lights array
   - Converts scene lights to shader format

## Status
Uniform structures now match shader expectations. Need to verify rendering works.

## Next Steps
1. Test Vulkan rendering with fixed uniforms
2. If still no output, check:
   - Matrix multiplication order (proj * view vs view * proj)
   - Vertex data format and content  
   - Viewport/scissor settings
   - Depth testing configuration
3. Once Vulkan works, apply same fixes to DirectX backend

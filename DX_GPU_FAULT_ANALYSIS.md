# DirectX GPU Fault Analysis

## Problem Summary
The DirectX backend is experiencing GPU faults when rendering, causing device loss with the error:
```
radv: GPUVM fault detected at address 0x00000000.
GCVM_L2_PROTECTION_FAULT_STATUS: 0x301430/0x501430
CLIENT_ID: (SQC (data)) 0xa
PERMISSION_FAULTS: 3
MAPPING_ERROR: 0
RW: 0
```

This indicates the GPU is trying to read from an invalid/null GPU address.

## Root Cause

### Descriptor Table Mismatch
The root signature defines a descriptor table at root parameter 3 with TWO consecutive texture descriptors:
- t0 (baseColorTexture) at offset 0
- t1 (shadowMap) at offset 1

From `dx12_impl.rs:1614-1630`:
```rust
let texture_descriptor_ranges = vec![
    // t0: baseColorTexture
    D3D12_DESCRIPTOR_RANGE {
        RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        NumDescriptors: 1,
        BaseShaderRegister: 0, // t0
        RegisterSpace: 0,
        OffsetInDescriptorsFromTableStart: 0,
    },
    // t1: shadowMap  
    D3D12_DESCRIPTOR_RANGE {
        RangeType: D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
        NumDescriptors: 1,
        BaseShaderRegister: 1, // t1
        RegisterSpace: 0,
        OffsetInDescriptorsFromTableStart: 1,  // <-- Expects next slot!
    },
];
```

### Only One SRV Created
However, we're only creating ONE SRV in the descriptor heap:
- Albedo texture gets SRV at heap offset 0 (GPU handle: 0x300000000)
- **No SRV is created at offset 1 for the shadow map**

From logs:
```
[2025-11-15 23:16:46.362] Created SRV at heap offset 0, GPU handle: 0x300000000, next offset: 1
[2025-11-15 23:16:46.427] Set descriptor table at root parameter 3, t0 handle: 0x300000000 (t1 at next slot)
```

### Shader Always Samples Both Textures
The HLSL shader in `shaders/hlsl/forward_simple.hlsl` samples from BOTH textures unconditionally:
```hlsl
Texture2D baseColorTexture : register(t0);
Texture2D shadowMap : register(t1);  // <-- Always accessed!
SamplerState baseColorSampler : register(s1);
SamplerComparisonState shadowSampler : register(s2);

// In PSMain():
float4 baseColor = baseColorTexture.Sample(baseColorSampler, input.texCoord);
// ... later ...
float shadow = shadowMap.SampleCmpLevelZero(shadowSampler, ...);  // <-- NULL pointer!
```

When the shader tries to sample from t1 (shadowMap), it's reading from an uninitialized descriptor at heap offset 1, which contains garbage or null, causing the GPU fault.

## Solutions

### Option 1: Create Default Shadow Map Texture (Recommended)
Always create a dummy 1x1 white texture for the shadow map at SRV heap offset 1, even when shadows are disabled. This ensures the descriptor table has valid descriptors at both offsets.

**Pros:**
- Simple and safe
- Shader doesn't need modification
- Consistent with how we handle albedo texture (default white texture)

**Cons:**
- Uses one extra descriptor slot
- Small memory overhead for dummy texture

### Option 2: Shader Branching Based on Uniforms
Add a uniform that indicates if shadows are enabled, and conditionally sample the shadow map:
```hlsl
cbuffer ShadowUniforms : register(b1) {
    float4x4 lightSpaceMatrix;
    bool shadowsEnabled;  // <-- Add this
    // ...
};

// In PSMain():
float shadow = 1.0;
if (shadowsEnabled) {
    shadow = shadowMap.SampleCmpLevelZero(...);
}
```

**Pros:**
- More "correct" - only samples when needed
- Potentially better performance when shadows disabled

**Cons:**
- Requires shader modification
- Still need a valid descriptor (can't leave t1 unbound)
- Branching in shader might hurt performance

### Option 3: Separate Root Signatures
Create two root signatures - one for forward passes with shadows, one without:
- Forward-only: Descriptor table with just t0
- Forward+Shadow: Descriptor table with t0 and t1

**Pros:**
- Most flexible
- Can optimize each case

**Cons:**
- Significantly more complex
- Need to manage multiple pipelines
- Overkill for this use case

## Recommended Fix: Option 1

1. In `app.rs::build_render_graph()`, always create a default shadow map texture:
```rust
// Create default 1x1 white shadow map (even if unused)
let default_shadow_map = Self::create_default_texture(&mut graph)?;
```

2. Ensure SRVs are created in order:
   - Albedo texture at heap offset 0
   - Shadow map at heap offset 1

3. Always bind both textures before draw:
   - bind_texture(0, 2, albedo_ptr)  // t0
   - bind_texture(0, 4, shadow_ptr)  // t1

This ensures the descriptor table always has valid descriptors at both required offsets.

## Additional Issues

### Logging Under Wine
Application logs were not visible in console output under Wine/Proton. Fixed by:
- Separating console and file logging dispatches in `main.rs`
- Removing duplicate stdout/stderr chains
- Logs now correctly write to `rusty_renderer.log`

### Performance Issues
The application is very slow between frames (~10 seconds), causing timeouts. This needs investigation but is likely due to:
- Excessive synchronization/waits
- Repeated resource allocation
- Missing command allocator reuse logic

## Next Steps
1. Implement Option 1 (default shadow map texture)
2. Test with both triangle and textured cube scenes
3. Investigate frame timing/performance issues
4. Verify proper descriptor heap management

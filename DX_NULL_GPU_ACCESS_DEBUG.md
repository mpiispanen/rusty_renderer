# DirectX NULL GPU Memory Access - Debug Analysis

## Problem Summary
DirectX backend crashes with GPU device lost error due to NULL memory access:
```
radv: GPUVM fault detected at address 0x00000000.
GCVM_L2_PROTECTION_FAULT_STATUS: 0x301430
CLIENT_ID: (SQC (data)) 0xa  <- Shader Queue Cache trying to read data
PERMISSION_FAULTS: 3
```

This indicates the shader is trying to access GPU memory at address 0x00000000 (NULL).

## What We Know Works
1. ✅ Texture creation with initial data works
2. ✅ SRV (Shader Resource View) descriptors are created
3. ✅ Upload to texture is implemented
4. ✅ Descriptor heap is set before draws
5. ✅ Vertex and index buffers are created and uploaded
6. ✅ Vulkan backend renders correctly

## Potential Causes

### 1. Descriptor Table Not Set Properly
The descriptor table at root parameter 3 contains both t0 (albedo) and t1 (shadow map).
Current logic only sets it when:
- `!self.descriptor_table_set` AND
- `self.bound_texture_t0.is_some()`

**Issue**: If we draw before binding any texture, descriptor_table_set stays false, and we never set the table!

### 2. Default Texture Not Bound
The forward_simple pass creates a default white texture when no texture is provided, but:
- Is this texture actually being bound to the pipeline?
- Is the SRV handle valid?

### 3. Shadow Map Texture May Be Null
The shader expects both t0 (albedo) and t1 (shadow map) in consecutive slots.
If we only bind t0 but shader samples from t1, we get NULL access.

### 4. Root Signature Mismatch
The root signature expects:
- Parameter 0: Camera uniforms (CBV b0)
- Parameter 1: Model matrix (32-bit constants)
- Parameter 2: Light uniform (CBV b1)  
- Parameter 3: Descriptor table (t0, t1, s0, s1)

If any mismatch exists between what we set and what shader expects = crash.

### 5. Resource State Transitions
Textures might not be in the correct state (D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE) when sampled.

## Debug Steps to Take

1. **Add logging in bind_texture** to see WHEN textures are bound
2. **Log descriptor_table_set state** before each draw
3. **Always bind default texture** if no texture provided, and ALWAYS set descriptor table
4. **Check if shadow map slot (t1) has valid handle** even if we don't use shadows
5. **Verify resource states** with validation layers (--debug flag)
6. **Add renderdoc/pix capture** to see actual GPU state

## Immediate Fix Attempts

### Option A: Always Bind Descriptor Table
Even if we don't have textures, bind descriptor table pointing to default/dummy descriptors.

### Option B: Shader-Side Fix  
Make shader handle missing textures (use #ifdef or default to white if sample fails).

### Option C: Guarantee Valid Descriptors
Always create both t0 and t1 descriptors, even if just pointing to 1x1 white texture.

## Next Actions
1. Add extensive logging around descriptor table setup
2. Run with DX debug layers enabled
3. Check if default texture's SRV handle is actually being used
4. Verify that draws happen AFTER textures are bound

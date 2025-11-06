# glTF Texture Rendering Fix

## Problem
The damaged helmet glTF model was rendering as a solid color without textures. Only the geometry was visible, and there were reports of artifacts (duplicate geometry appearing).

## Root Cause
The `forward_simple.hlsl` shader was only using vertex colors and not sampling any textures, despite the glTF loader properly extracting and storing material textures.

## Changes Made

### 1. Shader Updates (`shaders/hlsl/forward_simple.hlsl`)
- Added base color texture and sampler declarations at binding 2
- Updated vertex shader output to pass UV coordinates
- Modified pixel shader to sample the base color texture and multiply with vertex color
- Adjusted shadow map binding to slot 1 (from slot 0)

### 2. Application Updates (`src/app.rs`)
- Added `load_texture_to_graph()` helper method to load PNG/image files into render graph resources
- Modified `build_render_graph()` to:
  - Extract texture path from scene material
  - Load texture into render graph
  - Pass albedo texture to forward pass builder

### 3. Forward Pass Updates (`src/passes/forward_simple.rs`)
- Added albedo texture binding in the `execute()` callback
- Binds texture at set 0, binding 2 (matching shader declaration)

## Testing
The damaged helmet model (46,356 indices, 2048x2048 texture) now:
- Loads the embedded texture from glTF
- Extracts it to `.gltf_cache` directory
- Loads texture into GPU memory via render graph
- Binds it properly for sampling in the fragment shader
- Modulates texture color with vertex color for final surface appearance

## Known Limitations
1. **Single Material Support**: Currently only loads texture from the first object's first material
2. **Base Color Only**: Only supports base color/diffuse texture, not full PBR (metallic, roughness, normal maps)
3. **No Material System**: Textures are hardcoded per-scene rather than per-object/per-material
4. **Vertex Color Modulation**: Texture is multiplied by vertex color - if vertex colors are dark, texture appears dark

## Potential Artifacts
If you're still seeing "duplicate geometry" artifacts when moving the camera:
1. **Depth Testing**: The pass uses depth testing, so this shouldn't cause z-fighting
2. **Multiple Draw Calls**: Check if the scene accidentally has multiple objects with the same geometry
3. **Coordinate System**: Camera movement might reveal geometry from unexpected angles
4. **Shadow Map Sampling**: Errors in shadow mapping could create visual artifacts

## Next Steps
To fully support textured glTF models:
1. Implement per-material texture binding (render multiple objects with different textures)
2. Add support for additional PBR textures (metallic-roughness, normal, occlusion)
3. Create a material system that manages texture lifetimes
4. Add mipmap generation for better texture filtering
5. Support texture transforms and coordinate sets from glTF

## Usage
Run the damaged helmet scene:
```bash
cargo run --release -- --scene damaged_helmet
```

Move camera with WASD, look with mouse (click to capture), ESC to exit.

// Triangle shader for wgpu backend (WGSL)
// Renders a simple colored triangle with hardcoded vertices
// 
// NOTE: wgpu uses a different Y-axis convention than Vulkan:
// - Vulkan: Y points DOWN (NDC: -1 at top, +1 at bottom)
// - wgpu:   Y points UP   (NDC: +1 at top, -1 at bottom)
// We flip Y coordinates to match Vulkan's output

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// Vertex shader
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Hardcoded triangle vertices (NDC coordinates)
    // Y coordinates are flipped to match Vulkan output
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, 0.5),    // Bottom center (flipped from -0.5)
        vec2<f32>(0.5, -0.5),   // Top right (flipped from 0.5)
        vec2<f32>(-0.5, -0.5)   // Top left (flipped from 0.5)
    );
    
    // Hardcoded vertex colors (RGB)
    // Colors match geometric positions: Red at bottom, Green top-right, Blue top-left
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0),  // Red - bottom center (vertex 0)
        vec3<f32>(0.0, 1.0, 0.0),  // Green - top right (vertex 1)
        vec3<f32>(0.0, 0.0, 1.0)   // Blue - top left (vertex 2)
    );
    
    out.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    out.color = colors[vertex_index];
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

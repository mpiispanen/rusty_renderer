// Triangle shader for wgpu backend (WGSL)
// Renders a simple colored triangle with hardcoded vertices

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// Vertex shader
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Hardcoded triangle vertices (NDC coordinates)
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(0.0, -0.5),   // Bottom center
        vec2<f32>(-0.5, 0.5),   // Top left
        vec2<f32>(0.5, 0.5)     // Top right
    );
    
    // Hardcoded vertex colors (RGB)
    var colors = array<vec3<f32>, 3>(
        vec3<f32>(1.0, 0.0, 0.0),  // Red
        vec3<f32>(0.0, 1.0, 0.0),  // Green
        vec3<f32>(0.0, 0.0, 1.0)   // Blue
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

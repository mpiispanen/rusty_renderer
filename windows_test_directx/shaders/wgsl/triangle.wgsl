// Triangle shader for wgpu (hardcoded vertices)

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
}

// Hardcoded triangle vertices
const positions = array<vec2<f32>, 3>(
    vec2<f32>(0.0, -0.5),  // Bottom
    vec2<f32>(0.5, 0.5),   // Top Right  
    vec2<f32>(-0.5, 0.5)   // Top Left
);

const colors = array<vec3<f32>, 3>(
    vec3<f32>(1.0, 0.0, 0.0),  // Red
    vec3<f32>(0.0, 1.0, 0.0),  // Green
    vec3<f32>(0.0, 0.0, 1.0)   // Blue
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // Flip Y for wgpu coordinate system
    out.position = vec4<f32>(positions[vertex_index].x, -positions[vertex_index].y, 0.0, 1.0);
    out.color = colors[vertex_index];
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

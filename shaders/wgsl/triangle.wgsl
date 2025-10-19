// Triangle shader for wgpu backend (WGSL)
// Renders a simple colored triangle using vertex buffers
// 
// NOTE: wgpu uses a different Y-axis convention than Vulkan:
// - Vulkan: Y points DOWN (NDC: -1 at top, +1 at bottom)
// - wgpu:   Y points UP   (NDC: +1 at top, -1 at bottom)
// We flip Y coordinates to match Vulkan's output

// Vertex input from vertex buffer
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>,
}

// Vertex shader
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // For 2D triangle, we only use X and Y from position
    // Flip Y to match Vulkan convention
    out.position = vec4<f32>(in.position.x, -in.position.y, 0.0, 1.0);
    out.color = in.color.rgb;
    out.uv = in.uv;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

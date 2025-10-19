// Triangle shader for DirectX 12 backend (HLSL)
// Renders a simple colored triangle using vertex buffers

// Vertex input from vertex buffer
struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
    float2 uv : TEXCOORD0;
};

// Vertex Shader
VSOutput VSMain(VSInput input) {
    VSOutput output;
    
    // For 2D triangle, we only use X and Y from position
    // DirectX 12 uses Y-axis pointing UP (like wgpu, opposite to Vulkan)
    // Flip Y to match Vulkan convention
    output.position = float4(input.position.x, -input.position.y, 0.0, 1.0);
    output.color = input.color.rgb;
    output.uv = input.uv;
    
    return output;
}

// Pixel Shader
float4 PSMain(VSOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}

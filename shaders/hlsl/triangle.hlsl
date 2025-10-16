// Triangle shader for DirectX 12 backend (HLSL)
// Renders a simple colored triangle with hardcoded vertices

struct VSOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
};

// Vertex Shader
VSOutput VSMain(uint vertexID : SV_VertexID) {
    VSOutput output;
    
    // Hardcoded triangle vertices (NDC coordinates)
    // DirectX 12 uses same coordinate system as Vulkan (Y down)
    float2 positions[3] = {
        float2(0.0, -0.5),   // Bottom center
        float2(0.5, 0.5),    // Top right
        float2(-0.5, 0.5)    // Top left
    };
    
    // Hardcoded vertex colors (RGB)
    float3 colors[3] = {
        float3(1.0, 0.0, 0.0),  // Red
        float3(0.0, 1.0, 0.0),  // Green
        float3(0.0, 0.0, 1.0)   // Blue
    };
    
    output.position = float4(positions[vertexID], 0.0, 1.0);
    output.color = colors[vertexID];
    
    return output;
}

// Pixel Shader
float4 PSMain(VSOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}

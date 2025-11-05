// Shadow Map Shader (HLSL)
// Depth-only rendering from light's perspective

// Light view-projection matrix
cbuffer LightUniforms : register(b0) {
    float4x4 lightViewProj;
};

// Vertex input
struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

// Vertex output
struct PSInput {
    float4 position : SV_POSITION;
};

// Vertex Shader
PSInput VSMain(VSInput input) {
    PSInput output;
    
    // Transform directly to light clip space
    // For now, assume no model transform (identity matrix)
    output.position = mul(lightViewProj, float4(input.position, 1.0));
    
    return output;
}

// Pixel Shader (depth-only, no color output needed)
void PSMain(PSInput input) {
    // Depth is written automatically
    // No color output needed for shadow mapping
}

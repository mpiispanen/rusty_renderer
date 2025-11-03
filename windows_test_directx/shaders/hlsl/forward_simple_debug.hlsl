// Debug version - just output magenta for any vertex that makes it through

cbuffer CameraUniforms : register(b0) {
    float4x4 viewProj;
};

struct PushConstantData {
    float4x4 model;
    float4x4 normalMatrix;
};
[[vk::push_constant]] PushConstantData pushConstants;

struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

struct PSInput {
    float4 position : SV_POSITION;
    float4 color : COLOR0;
};

PSInput VSMain(VSInput input) {
    PSInput output;
    
    // Simple passthrough - just transform position
    float4 worldPos = mul(pushConstants.model, float4(input.position, 1.0));
    output.position = mul(viewProj, worldPos);
    output.color = input.color;
    
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    return float4(1.0, 0.0, 1.0, 1.0); // Magenta
}

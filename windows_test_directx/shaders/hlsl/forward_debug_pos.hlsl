// Debug shader - output position as color

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
    float4 debugColor : COLOR0;
};

PSInput VSMain(VSInput input) {
    PSInput output;
    
    // Transform position
    float4 worldPos = mul(float4(input.position, 1.0), pushConstants.model);
    output.position = mul(worldPos, viewProj);
    
    // Output position as color (map from [-1,1] to [0,1])
    output.debugColor = float4(input.position * 0.5 + 0.5, 1.0);
    
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    return input.debugColor;
}

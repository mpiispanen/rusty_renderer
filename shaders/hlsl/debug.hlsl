// Debug shader to visualize data
cbuffer CameraUniforms : register(b0) {
    float4x4 viewProj;
};

cbuffer PushConstants : register(b2) {
    float4x4 model;
    float4x4 normalMatrix;
};

struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

struct PSInput {
    float4 position : SV_POSITION;
    float3 worldPos : POSITION0;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

PSInput VSMain(VSInput input) {
    PSInput output;
    float4 worldPos = mul(model, float4(input.position, 1.0));
    output.worldPos = worldPos.xyz;
    output.normal = mul((float3x3)normalMatrix, input.normal);
    output.uv = input.uv;
    output.color = input.color;
    output.position = mul(viewProj, worldPos);
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    // Debug output: Show vertex color directly
    return input.color;
    
    // Alternative debug outputs (uncomment to test):
    // return float4(input.normal * 0.5 + 0.5, 1.0); // Normals
    // return float4(input.uv, 0.0, 1.0); // UVs
    // return float4(1.0, 0.0, 0.0, 1.0); // Solid red
}

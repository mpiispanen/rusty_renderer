// Test shader - output UV as color
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
    // Output UV as color (red=U, green=V, blue=1)
    return float4(input.uv.x, input.uv.y, 0.0, 1.0);
}

// Minimal test shader for DirectX/vkd3d-proton compatibility testing

cbuffer Constants : register(b0) {
    float4x4 mvp;
};

struct VSInput {
    float3 position : POSITION;
    float4 color : COLOR0;
};

struct PSInput {
    float4 position : SV_POSITION;
    float4 color : COLOR0;
};

PSInput VSMain(VSInput input) {
    PSInput output;
    output.position = mul(mvp, float4(input.position, 1.0));
    output.color = input.color;
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    return input.color;
}

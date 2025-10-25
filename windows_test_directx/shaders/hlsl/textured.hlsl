// Textured shader for DirectX 12 backend

struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 texCoord : TEXCOORD0;
    float4 color : COLOR;
};

struct PSInput {
    float4 position : SV_POSITION;
    float2 texCoord : TEXCOORD0;
    float4 color : COLOR;
};

// Texture and sampler
Texture2D g_texture : register(t0);
SamplerState g_sampler : register(s0);

PSInput VSMain(VSInput input) {
    PSInput output;
    output.position = float4(input.position, 1.0f);
    output.texCoord = input.texCoord;
    output.color = input.color;
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    float4 texColor = g_texture.Sample(g_sampler, input.texCoord);
    return texColor * input.color;
}

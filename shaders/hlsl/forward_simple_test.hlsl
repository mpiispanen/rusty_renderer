// Ultra-simple test - no transforms, just pass through NDC coordinates

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
    
    // Just pass position through as if it's already in NDC (clip space)
    // Scale it down so a cube from -0.5 to 0.5 fits in -1 to 1 NDC
    output.position = float4(input.position.xy * 2.0, 0.0, 1.0);
    output.color = input.color;
    
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    return input.color; // Return vertex color
}

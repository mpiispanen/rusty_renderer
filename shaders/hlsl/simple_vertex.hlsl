// Simple vertex-colored shader that reads from vertex buffers
// Used for basic vertex buffer rendering

struct VSInput {
    float2 position : POSITION;
    float3 color : COLOR;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
};

VSOutput VSMain(VSInput input) {
    VSOutput output;
#ifdef VULKAN
    output.position = float4(input.position, 0.0, 1.0);
#else
    // Flip Y for DirectX coordinate system
    output.position = float4(input.position.x, -input.position.y, 0.0, 1.0);
#endif
    output.color = input.color;
    return output;
}

float4 PSMain(VSOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}

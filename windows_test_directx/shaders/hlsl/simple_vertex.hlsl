// Simple vertex-colored shader that reads from vertex buffers
// Used for basic vertex buffer rendering

// Push constants for transformation matrices
struct PushConstants {
    float4x4 mvp;
};

[[vk::push_constant]]
PushConstants pushConstants;

struct VSInput {
    float3 position : POSITION;
    float3 color : COLOR;
};

struct VSOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
};

VSOutput VSMain(VSInput input) {
    VSOutput output;
    
    // Transform position by MVP matrix
    float4 worldPos = float4(input.position, 1.0);
    output.position = mul(pushConstants.mvp, worldPos);
    
    output.color = input.color;
    return output;
}

float4 PSMain(VSOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}

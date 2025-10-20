// Triangle shader for DirectX 12 (hardcoded vertices)

struct VSOutput {
    float4 position : SV_POSITION;
    float3 color : COLOR0;
};

// Hardcoded triangle vertices
static float2 positions[3] = {
    float2(0.0, -0.5),  // Bottom
    float2(0.5, 0.5),   // Top Right
    float2(-0.5, 0.5)   // Top Left
};

static float3 colors[3] = {
    float3(1.0, 0.0, 0.0),  // Red
    float3(0.0, 1.0, 0.0),  // Green
    float3(0.0, 0.0, 1.0)   // Blue
};

VSOutput VSMain(uint vertexID : SV_VertexID) {
    VSOutput output;
    // Flip Y for DirectX coordinate system
    output.position = float4(positions[vertexID].x, -positions[vertexID].y, 0.0, 1.0);
    output.color = colors[vertexID];
    return output;
}

float4 PSMain(VSOutput input) : SV_TARGET {
    return float4(input.color, 1.0);
}

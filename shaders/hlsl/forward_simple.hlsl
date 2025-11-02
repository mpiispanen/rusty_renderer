// Simplified forward rendering shader for DirectX 12
// No textures - just vertex colors and basic lighting

#define MAX_LIGHTS 8
#define LIGHT_DIRECTIONAL 0
#define LIGHT_POINT 1

// Camera uniforms (b0)
cbuffer CameraUniforms : register(b0) {
    float4x4 viewProj;
};

// Lighting uniforms (b1)
struct Light {
    uint lightType;
    uint padding1;
    uint padding2;
    uint padding3;
    float4 positionOrDirection;
    float4 colorIntensity;
};

cbuffer LightingUniforms : register(b1) {
    float4 ambientLightCount;
    Light lights[MAX_LIGHTS];
};

// Push constants struct
struct PushConstantData {
    float4x4 model;
    float4x4 normalMatrix;
};
[[vk::push_constant]] PushConstantData pushConstants;

// Vertex input
struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

// Vertex shader output
struct PSInput {
    float4 position : SV_POSITION;
    float3 worldPos : POSITION0;
    float3 normal : NORMAL;
    float4 color : COLOR0;
};

// Vertex Shader
PSInput VSMain(VSInput input) {
    PSInput output;
    
    // Transform to world space
    float4 worldPos = mul(pushConstants.model, float4(input.position, 1.0));
    output.worldPos = worldPos.xyz;
    
    // Transform normal
    output.normal = normalize(mul((float3x3)pushConstants.normalMatrix, input.normal));
    
    // Transform to clip space
    output.position = mul(viewProj, worldPos);
    
    // Pass color
    output.color = input.color;
    
    return output;
}

// Pixel Shader with simple lighting
float4 PSMain(PSInput input) : SV_TARGET {
    // Normalize interpolated normal
    float3 normal = normalize(input.normal);
    
    // Use vertex color (no material for now)
    float3 surfaceColor = input.color.rgb;
    
    // Start with ambient light
    float3 ambient = ambientLightCount.xyz;
    float3 finalColor = ambient * surfaceColor;
    
    // Add contribution from each light
    uint lightCount = (uint)ambientLightCount.w;
    for (uint i = 0; i < lightCount && i < MAX_LIGHTS; i++) {
        Light light = lights[i];
        
        float3 lightDir;
        float attenuation = 1.0;
        
        if (light.lightType == LIGHT_DIRECTIONAL) {
            // Directional light
            lightDir = normalize(-light.positionOrDirection.xyz);
        } else {
            // Point light
            float3 lightVec = light.positionOrDirection.xyz - input.worldPos;
            float distance = length(lightVec);
            lightDir = lightVec / distance;
            attenuation = 1.0 / (1.0 + 0.1 * distance + 0.01 * distance * distance);
        }
        
        // Diffuse lighting
        float diff = max(dot(normal, lightDir), 0.0);
        float3 diffuse = diff * light.colorIntensity.rgb * light.colorIntensity.a;
        
        finalColor += diffuse * surfaceColor * attenuation;
    }
    
    return float4(finalColor, 1.0);
}

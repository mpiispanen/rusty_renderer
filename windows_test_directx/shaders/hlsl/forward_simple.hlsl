// Simplified forward rendering shader for DirectX 12
// Supports vertex colors, basic lighting, and optional base color texture

#define MAX_LIGHTS 8
#define LIGHT_DIRECTIONAL 0
#define LIGHT_POINT 1

// Lighting uniforms (b0)
struct Light {
    uint lightType;
    uint padding1;
    uint padding2;
    uint padding3;
    float4 positionOrDirection;
    float4 colorIntensity;
};

cbuffer LightingUniforms : register(b0) {
    float4 ambientLightCount;
    Light lights[MAX_LIGHTS];
};

// Shadow uniforms (b1)
#ifdef VULKAN
[[vk::binding(1, 0)]]
#endif
cbuffer ShadowUniforms : register(b1) {
    float4x4 lightSpaceMatrix;
    float4 shadowParams; // x: enabled (0 or 1), y: bias, z: unused, w: unused
};

// Push constants struct (b2 for DirectX, vk::push_constant for Vulkan)
// Now includes camera matrices for dynamic updates
struct PushConstantData {
    float4x4 viewProj;      // Camera view-projection matrix
    float4x4 model;         // Model transform matrix
    float4x4 normalMatrix;  // Normal matrix (inverse transpose of model)
};

#ifdef VULKAN
[[vk::push_constant]] PushConstantData pushConstants;
#else
// DirectX uses root constants at b2
cbuffer PushConstants : register(b2) {
    PushConstantData pushConstants;
};
#endif

// Base color texture (t0) and sampler (s0) - optional material texture
#ifdef VULKAN
[[vk::binding(2, 0)]]
Texture2D baseColorTexture : register(t0);
[[vk::binding(2, 0)]]
SamplerState baseColorSampler : register(s0);
#else
Texture2D baseColorTexture : register(t0);
SamplerState baseColorSampler : register(s0);
#endif

// Shadow map texture (t1) - only if ENABLE_SHADOWS is defined
#if defined(ENABLE_SHADOWS)
#ifdef VULKAN
[[vk::binding(4, 0)]]
Texture2D shadowMap : register(t1);
[[vk::binding(4, 0)]]
SamplerComparisonState shadowSampler : register(s2);
#else
Texture2D shadowMap : register(t1);
SamplerComparisonState shadowSampler : register(s2);
#endif
#endif

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
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
    float4 lightSpacePos : POSITION1; // For shadow mapping
};

// Vertex Shader
PSInput VSMain(VSInput input) {
    PSInput output;
    
    // Transform to world space
    float4 worldPos = mul(pushConstants.model, float4(input.position, 1.0));
    output.worldPos = worldPos.xyz;
    
    // Transform normal
    output.normal = normalize(mul((float3x3)pushConstants.normalMatrix, input.normal));
    
    // Transform to clip space using push constant view-proj matrix
    output.position = mul(pushConstants.viewProj, worldPos);
    
    // Transform to light space for shadow mapping
    output.lightSpacePos = mul(lightSpaceMatrix, worldPos);
    
    // Pass UV coordinates and color
    output.uv = input.uv;
    output.color = input.color;
    
    return output;
}

// Pixel Shader with simple lighting and shadow calculation
#if defined(ENABLE_SHADOWS)
float CalculateShadow(float4 lightSpacePos, float3 normal, float3 lightDir) {
    // Perspective divide
    float3 projCoords = lightSpacePos.xyz / lightSpacePos.w;
    
    // Transform to [0,1] range (from NDC [-1,1])
    projCoords.xy = projCoords.xy * 0.5 + 0.5;
    
    // Flip Y for Vulkan/D3D coordinate system
    projCoords.y = 1.0 - projCoords.y;
    
    // Check if outside shadow map
    if (projCoords.x < 0.0 || projCoords.x > 1.0 || 
        projCoords.y < 0.0 || projCoords.y > 1.0 || 
        projCoords.z > 1.0) {
        return 1.0; // No shadow
    }
    
    // Bias to reduce shadow acne
    float bias = max(shadowParams.y * (1.0 - dot(normal, lightDir)), shadowParams.y * 0.1);
    float currentDepth = projCoords.z - bias;
    
    // PCF (Percentage Closer Filtering)
    float shadow = 0.0;
    float2 texelSize = 1.0 / float2(1024.0, 1024.0); // TODO: pass actual shadow map size
    
    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float2 offset = float2(x, y) * texelSize;
            shadow += shadowMap.SampleCmpLevelZero(shadowSampler, projCoords.xy + offset, currentDepth);
        }
    }
    shadow /= 9.0;
    
    return shadow;
}
#endif

float4 PSMain(PSInput input) : SV_TARGET {
    // Normalize interpolated normal
    float3 normal = normalize(input.normal);
    
    // Sample base color texture and modulate with vertex color
    // If texture is white/default, vertex color dominates
    // If vertex color is white/default, texture dominates
    float3 textureColor = baseColorTexture.Sample(baseColorSampler, input.uv).rgb;
    float3 surfaceColor = textureColor * input.color.rgb;
    
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
        
        // Apply shadow for directional lights
        float shadow = 1.0;
#if defined(ENABLE_SHADOWS)
        if (light.lightType == LIGHT_DIRECTIONAL && shadowParams.x > 0.5) {
            shadow = CalculateShadow(input.lightSpacePos, normal, lightDir);
        }
#endif
        
        finalColor += diffuse * surfaceColor * attenuation * shadow;
    }
    
    return float4(finalColor, 1.0);
}

// Forward rendering shader - SIMPLIFIED FOR DEBUGGING
#define MAX_LIGHTS 8
#define LIGHT_DIRECTIONAL 0
#define LIGHT_POINT 1

cbuffer CameraUniforms : register(b0) {
    float4x4 viewProj;
};

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

// Push constants: use Vulkan attribute when compiling to SPIR-V
// DirectX runtime compiler will ignore the [[vk::push_constant]] attribute
#ifdef VULKAN
[[vk::push_constant]]
#endif
cbuffer PushConstants
#ifndef VULKAN
: register(b2)
#endif
{
    float4x4 model;
    float4x4 normalMatrix;
};

cbuffer MaterialUniforms : register(b3) {
    float4 baseColor;
    float4 properties; // x = metallic, y = roughness, z = hasTexture
};

// Explicitly specify Vulkan bindings for texture and sampler
[[vk::binding(2, 0)]]
Texture2D diffuseTexture : register(t0);
[[vk::binding(2, 0)]]
SamplerState diffuseSampler : register(s0);

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
    
    // UV handling: DirectX has V=0 at top, Vulkan has V=0 at bottom
    #ifdef VULKAN
    output.uv = input.uv;  // No flip for Vulkan
    #else
    output.uv = float2(input.uv.x, 1.0 - input.uv.y);  // Flip V for DirectX
    #endif
    
    output.color = input.color;
    output.position = mul(viewProj, worldPos);
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    // Start with base color
    float3 albedo = baseColor.rgb;
    
    // Sample and apply texture if available  
    if (properties.z > 0.5) {
        float4 texColor = diffuseTexture.Sample(diffuseSampler, input.uv);
        albedo *= texColor.rgb;
    }
    
    // Blend with vertex color
    albedo *= input.color.rgb;
    
    // Normalize the normal
    float3 normal = normalize(input.normal);
    
    // View direction (assuming camera at origin - matches Vulkan shader)
    float3 viewDir = normalize(-input.worldPos);
    
    // Material properties (match Vulkan shader)
    const float shininess = 32.0;
    const float specularStrength = 0.5;
    
    // Start with ambient light
    float3 ambient = ambientLightCount.xyz;
    float3 finalColor = ambient * albedo;
    
    // Accumulate light contributions
    int lightCount = (int)ambientLightCount.w;
    for (int i = 0; i < lightCount && i < MAX_LIGHTS; i++) {
        float3 lightDir;
        float attenuation = 1.0;
        
        if (lights[i].lightType == LIGHT_DIRECTIONAL) {
            // Directional light
            lightDir = normalize(-lights[i].positionOrDirection.xyz);
        } else if (lights[i].lightType == LIGHT_POINT) {
            // Point light
            float3 lightPos = lights[i].positionOrDirection.xyz;
            float3 toLight = lightPos - input.worldPos;
            float distance = length(toLight);
            lightDir = normalize(toLight);
            
            // Attenuation (inverse square law with minimum)
            attenuation = 1.0 / max(distance * distance, 0.01);
        }
        
        // Diffuse (Lambertian)
        float diff = max(dot(normal, lightDir), 0.0);
        float3 diffuse = diff * lights[i].colorIntensity.rgb;
        
        // Specular (Blinn-Phong) - NOW MATCHES VULKAN
        float3 halfwayDir = normalize(lightDir + viewDir);
        float spec = pow(max(dot(normal, halfwayDir), 0.0), shininess);
        float3 specular = specularStrength * spec * lights[i].colorIntensity.rgb;
        
        // Apply intensity and attenuation
        float intensity = lights[i].colorIntensity.a;
        finalColor += (diffuse + specular) * intensity * attenuation * albedo;
    }
    
    return float4(finalColor, 1.0);
}


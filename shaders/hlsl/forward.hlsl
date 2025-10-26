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

cbuffer PushConstants : register(b2) {
    float4x4 model;
    float4x4 normalMatrix;
};

cbuffer MaterialUniforms : register(b3) {
    float4 baseColor;
    float4 properties; // x = metallic, y = roughness, z = hasTexture
};

Texture2D diffuseTexture : register(t0);
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
    output.uv = input.uv;
    output.color = input.color;
    output.position = mul(viewProj, worldPos);
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    // Start with base color
    float3 color = baseColor.rgb;
    
    // Sample and apply texture if available  
    if (properties.z > 0.5) {
        float4 texColor = diffuseTexture.Sample(diffuseSampler, input.uv);
        color *= texColor.rgb;
    }
    
    // Blend with vertex color
    color *= input.color.rgb;
    
    // Simple ambient lighting
    float3 ambient = ambientLightCount.xyz;
    float3 finalColor = ambient * color;
    
    // Add a simple diffuse term for the first directional light
    int lightCount = (int)ambientLightCount.w;
    if (lightCount > 0 && lights[0].lightType == LIGHT_DIRECTIONAL) {
        float3 lightDir = normalize(-lights[0].positionOrDirection.xyz);
        float3 normal = normalize(input.normal);
        float diff = max(dot(normal, lightDir), 0.0);
        finalColor += diff * lights[0].colorIntensity.rgb * lights[0].colorIntensity.a * color;
    }
    
    return float4(finalColor, 1.0);
}

// Minimal forward rendering shader for testing
#define MAX_LIGHTS 8
#define LIGHT_DIRECTIONAL 0
#define LIGHT_POINT 1

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

cbuffer ShadowUniforms : register(b1) {
    float4x4 lightSpaceMatrix;
    float4 shadowParams;
};

struct PushConstantData {
    float4x4 viewProj;
    float4x4 model;
    float4x4 normalMatrix;
};

cbuffer PushConstants : register(b2) {
    PushConstantData pushConstants;
};

Texture2D baseColorTexture : register(t0);
SamplerState baseColorSampler : register(s1);

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
    float4 worldPos = mul(pushConstants.model, float4(input.position, 1.0));
    output.worldPos = worldPos.xyz;
    output.normal = normalize(mul((float3x3)pushConstants.normalMatrix, input.normal));
    output.position = mul(pushConstants.viewProj, worldPos);
    output.uv = input.uv;
    output.color = input.color;
    return output;
}

float4 PSMain(PSInput input) : SV_TARGET {
    float3 normal = normalize(input.normal);
    float3 textureColor = baseColorTexture.Sample(baseColorSampler, input.uv).rgb;
    float3 surfaceColor = textureColor * input.color.rgb;
    
    float3 ambient = ambientLightCount.xyz;
    float3 finalColor = ambient * surfaceColor;
    
    uint lightCount = (uint)ambientLightCount.w;
    for (uint i = 0; i < lightCount && i < MAX_LIGHTS; i++) {
        Light light = lights[i];
        
        float3 lightDir;
        float attenuation = 1.0;
        
        if (light.lightType == LIGHT_DIRECTIONAL) {
            lightDir = normalize(-light.positionOrDirection.xyz);
        } else {
            float3 lightVec = light.positionOrDirection.xyz - input.worldPos;
            float distance = length(lightVec);
            lightDir = lightVec / distance;
            attenuation = 1.0 / (1.0 + 0.1 * distance + 0.01 * distance * distance);
        }
        
        float diff = max(dot(normal, lightDir), 0.0);
        float3 diffuse = diff * light.colorIntensity.rgb * light.colorIntensity.a;
        
        finalColor += diffuse * surfaceColor * attenuation;
    }
    
    return float4(finalColor, 1.0);
}

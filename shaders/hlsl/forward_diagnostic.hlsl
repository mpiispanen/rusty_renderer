// Forward rendering shader - DIAGNOSTIC VERSION
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
    // DIAGNOSTIC: Uncomment one of these lines to test different aspects
    
    // Test 1: Is shader running? (should see magenta)
    // return float4(1.0, 0.0, 1.0, 1.0);
    
    // Test 2: Are UVs correct? (should see gradient)
    // return float4(input.uv, 0.0, 1.0);
    
    // Test 3: Are normals being passed? (should see colored cube faces)
    // return float4(abs(normalize(input.normal)), 1.0);
    
    // Test 4: Is texture flag set? (magenta = yes, cyan = no)
    // return properties.z > 0.5 ? float4(1.0, 0.0, 1.0, 1.0) : float4(0.0, 1.0, 1.0, 1.0);
    
    // Test 5: What is the base color? (should see material base color)
    // return float4(baseColor.rgb, 1.0);
    
    // Test 6: What does the texture look like?
    // return diffuseTexture.Sample(diffuseSampler, input.uv);
    
    // Test 7: What are the vertex colors? (should see per-vertex colors if any)
    // return input.color;
    
    // NORMAL RENDERING CODE:
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
    
    // Start with ambient light
    float3 ambient = ambientLightCount.xyz;
    float3 finalColor = ambient * albedo;
    
    // Accumulate light contributions
    int lightCount = (int)ambientLightCount.w;
    for (int i = 0; i < lightCount && i < MAX_LIGHTS; i++) {
        if (lights[i].lightType == LIGHT_DIRECTIONAL) {
            // Directional light
            float3 lightDir = normalize(-lights[i].positionOrDirection.xyz);
            float diff = max(dot(normal, lightDir), 0.0);
            float3 diffuse = diff * lights[i].colorIntensity.rgb;
            float intensity = lights[i].colorIntensity.a;
            finalColor += diffuse * intensity * albedo;
        } else if (lights[i].lightType == LIGHT_POINT) {
            // Point light
            float3 lightPos = lights[i].positionOrDirection.xyz;
            float3 toLight = lightPos - input.worldPos;
            float distance = length(toLight);
            float3 lightDir = normalize(toLight);
            
            // Attenuation
            float attenuation = 1.0 / max(distance * distance, 0.01);
            
            // Diffuse
            float diff = max(dot(normal, lightDir), 0.0);
            float3 diffuse = diff * lights[i].colorIntensity.rgb;
            float intensity = lights[i].colorIntensity.a;
            finalColor += diffuse * intensity * attenuation * albedo;
        }
    }
    
    return float4(finalColor, 1.0);
}

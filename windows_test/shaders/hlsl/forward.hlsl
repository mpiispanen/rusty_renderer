// Forward rendering shader for DirectX 12 with lighting
// Matches forward.vert and forward.frag GLSL shaders

// Maximum number of lights (must match Rust code)
#define MAX_LIGHTS 8

// Light types
#define LIGHT_DIRECTIONAL 0
#define LIGHT_POINT 1

// Constant buffers (match root signature)

// Root parameter 0: Camera uniforms (b0)
cbuffer CameraUniforms : register(b0) {
    float4x4 viewProj;  // View-projection matrix
};

// Root parameter 1: Lighting uniforms (b1)
struct Light {
    uint lightType;
    uint padding1;
    uint padding2;
    uint padding3;
    float4 positionOrDirection;  // xyz = position/direction, w = 0 for directional, 1 for point
    float4 colorIntensity;       // rgb = color, a = intensity
};

cbuffer LightingUniforms : register(b1) {
    float4 ambientLightCount;  // xyz = ambient color, w = light count
    Light lights[MAX_LIGHTS];
};

// Root parameter 2: Push constants (root constants b2)
cbuffer PushConstants : register(b2) {
    float4x4 model;        // Model matrix
    float4x4 normalMatrix; // Normal matrix (inverse transpose of model)
};

// Material uniforms (b3)
cbuffer MaterialUniforms : register(b3) {
    float4 baseColor;      // rgb = base color, a = alpha
    float4 properties;     // x = metallic, y = roughness, z = hasTexture, w = padding
};

// Texture and sampler
Texture2D diffuseTexture : register(t0);
SamplerState diffuseSampler : register(s0);

// Vertex input structure
struct VSInput {
    float3 position : POSITION;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

// Vertex shader output / Pixel shader input
struct PSInput {
    float4 position : SV_POSITION;
    float3 worldPos : POSITION0;
    float3 normal : NORMAL;
    float2 uv : TEXCOORD0;
    float4 color : COLOR0;
};

// Vertex Shader
PSInput VSMain(VSInput input) {
    PSInput output;
    
    // Transform position to world space
    float4 worldPos = mul(model, float4(input.position, 1.0));
    output.worldPos = worldPos.xyz;
    
    // Transform normal to world space (using normal matrix to handle non-uniform scaling)
    output.normal = mul((float3x3)normalMatrix, input.normal);
    
    // Pass through UV and color
    output.uv = input.uv;
    output.color = input.color;
    
    // Transform to clip space
    output.position = mul(viewProj, worldPos);
    
    return output;
}

// Material properties
static const float shininess = 32.0;
static const float specularStrength = 0.5;

// Calculate lighting from a single light source
float3 calculateLight(Light light, float3 fragPosition, float3 normal, float3 viewDir) {
    float3 lightDir;
    float attenuation = 1.0;
    
    if (light.lightType == LIGHT_DIRECTIONAL) {
        // Directional light
        lightDir = normalize(-light.positionOrDirection.xyz);
    } else {
        // Point light
        float3 lightPos = light.positionOrDirection.xyz;
        float3 toLight = lightPos - fragPosition;
        float distance = length(toLight);
        lightDir = normalize(toLight);
        
        // Attenuation (inverse square law with minimum)
        attenuation = 1.0 / max(distance * distance, 0.01);
    }
    
    // Diffuse (Lambertian)
    float diff = max(dot(normal, lightDir), 0.0);
    float3 diffuse = diff * light.colorIntensity.rgb;
    
    // Specular (Blinn-Phong)
    float3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), shininess);
    float3 specular = specularStrength * spec * light.colorIntensity.rgb;
    
    // Apply intensity and attenuation
    float intensity = light.colorIntensity.a;
    return (diffuse + specular) * intensity * attenuation;
}

// Pixel Shader
float4 PSMain(PSInput input) : SV_TARGET {
    // Normalize interpolated normal
    float3 normal = normalize(input.normal);
    
    // View direction (assuming camera at origin for now)
    // TODO: Pass camera position as uniform
    float3 viewDir = normalize(-input.worldPos);
    
    // Get base color from material
    float3 color = baseColor.rgb;
    
    // Sample texture if available
    if (properties.z > 0.5) {  // hasTexture flag
        float4 texColor = diffuseTexture.Sample(diffuseSampler, input.uv);
        color *= texColor.rgb;  // Modulate base color with texture
    }
    
    // Blend with vertex color
    color *= input.color.rgb;
    
    // Start with ambient light
    float3 ambient = ambientLightCount.xyz * color;
    
    // Accumulate light contributions
    float3 lighting = ambient;
    int lightCount = (int)ambientLightCount.w;
    
    for (int i = 0; i < lightCount && i < MAX_LIGHTS; i++) {
        float3 lightContrib = calculateLight(lights[i], input.worldPos, normal, viewDir);
        lighting += lightContrib * color;
    }
    
    // Final color with alpha
    return float4(lighting, input.color.a);
}

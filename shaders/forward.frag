#version 450

// Fragment shader for forward rendering with Blinn-Phong lighting

// Maximum number of lights (must match Rust code)
#define MAX_LIGHTS 8

// Light types
#define LIGHT_DIRECTIONAL 0
#define LIGHT_POINT 1

// Light data structure (matches GPU layout in Rust)
struct Light {
    uint lightType;
    uint padding1;
    uint padding2;
    uint padding3;
    vec4 positionOrDirection;  // xyz = position/direction, w = 0 for directional, 1 for point
    vec4 colorIntensity;       // rgb = color, a = intensity
};

// Lighting uniforms
layout(set = 0, binding = 1) uniform LightingUniforms {
    vec4 ambientLightCount;  // xyz = ambient color, w = light count
    Light lights[MAX_LIGHTS];
} lighting;

// Texture sampler (optional - for textured materials)
layout(set = 0, binding = 2) uniform sampler2D diffuseTexture;

// Material properties
layout(set = 0, binding = 3) uniform MaterialUniforms {
    vec4 baseColor;          // rgb = base color, a = alpha
    vec4 properties;         // x = metallic, y = roughness, z = hasTexture, w = padding
} material;

// Inputs from vertex shader
layout(location = 0) in vec3 fragPosition;  // World space
layout(location = 1) in vec3 fragNormal;    // World space
layout(location = 2) in vec2 fragUV;
layout(location = 3) in vec4 fragColor;

// Output
layout(location = 0) out vec4 outColor;

// Material properties (TODO: make these uniforms)
const float shininess = 32.0;
const float specularStrength = 0.5;

// Calculate lighting from a single light source
vec3 calculateLight(Light light, vec3 normal, vec3 viewDir) {
    vec3 lightDir;
    float attenuation = 1.0;
    
    if (light.lightType == LIGHT_DIRECTIONAL) {
        // Directional light
        lightDir = normalize(-light.positionOrDirection.xyz);
    } else {
        // Point light
        vec3 lightPos = light.positionOrDirection.xyz;
        vec3 toLight = lightPos - fragPosition;
        float distance = length(toLight);
        lightDir = normalize(toLight);
        
        // Attenuation (inverse square law with minimum)
        attenuation = 1.0 / max(distance * distance, 0.01);
    }
    
    // Diffuse (Lambertian)
    float diff = max(dot(normal, lightDir), 0.0);
    vec3 diffuse = diff * light.colorIntensity.rgb;
    
    // Specular (Blinn-Phong)
    vec3 halfwayDir = normalize(lightDir + viewDir);
    float spec = pow(max(dot(normal, halfwayDir), 0.0), shininess);
    vec3 specular = specularStrength * spec * light.colorIntensity.rgb;
    
    // Apply intensity and attenuation
    float intensity = light.colorIntensity.a;
    return (diffuse + specular) * intensity * attenuation;
}

void main() {
    // Normalize interpolated normal
    vec3 normal = normalize(fragNormal);
    
    // View direction (assuming camera at origin for now)
    // TODO: Pass camera position as uniform
    vec3 viewDir = normalize(-fragPosition);
    
    // Use vertex color as base color (TODO: add material uniforms and textures)
    vec3 baseColor = fragColor.rgb;
    
    // Start with ambient light
    vec3 ambient = lighting.ambientLightCount.xyz * baseColor;
    
    // Accumulate light contributions
    vec3 lighting_result = ambient;
    int lightCount = int(lighting.ambientLightCount.w);
    
    for (int i = 0; i < lightCount && i < MAX_LIGHTS; i++) {
        vec3 lightContrib = calculateLight(lighting.lights[i], normal, viewDir);
        lighting_result += lightContrib * baseColor;
    }
    
    // Final color with alpha
    outColor = vec4(lighting_result, fragColor.a);
}

#version 450

// Simplified forward rendering fragment shader (no textures/materials)

// Lighting uniforms (simplified)
layout(set = 0, binding = 1) uniform LightingUniforms {
    vec3 ambient;
    float _padding1;
    vec3 lightDir;
    float _padding2;
    vec3 lightColor;
    float lightIntensity;
} lighting;

// Inputs from vertex shader
layout(location = 0) in vec3 fragPosition;  // World space
layout(location = 1) in vec3 fragNormal;    // World space
layout(location = 2) in vec2 fragUV;
layout(location = 3) in vec4 fragColor;

// Output
layout(location = 0) out vec4 outColor;

void main() {
    // Normalize interpolated normal
    vec3 normal = normalize(fragNormal);
    
    // Calculate simple directional lighting
    vec3 lightDir = normalize(-lighting.lightDir);  // Negate because shader expects direction TO light
    float diff = max(dot(normal, lightDir), 0.0);
    
    // Combine ambient and diffuse
    vec3 ambient = lighting.ambient;
    vec3 diffuse = diff * lighting.lightColor * lighting.lightIntensity;
    
    // Apply lighting to vertex color
    vec3 color = (ambient + diffuse) * fragColor.rgb;
    
    // Output final color
    outColor = vec4(color, fragColor.a);
}

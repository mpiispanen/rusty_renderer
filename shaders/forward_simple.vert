#version 450

// Simplified forward rendering vertex shader (no textures/materials)

// Vertex inputs (from vertex buffer)
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;
layout(location = 3) in vec4 inColor;

// Uniforms (camera transforms)
layout(set = 0, binding = 0) uniform CameraUniforms {
    mat4 viewProj;  // View-projection matrix
} camera;

// Push constants (per-object transforms)
layout(push_constant) uniform PushConstants {
    mat4 model;     // Model matrix
    mat4 normalMatrix;  // Normal matrix (inverse transpose of model)
} push;

// Outputs to fragment shader
layout(location = 0) out vec3 fragPosition;  // World space position
layout(location = 1) out vec3 fragNormal;    // World space normal
layout(location = 2) out vec2 fragUV;
layout(location = 3) out vec4 fragColor;

void main() {
    // Transform position to world space
    vec4 worldPos = push.model * vec4(inPosition, 1.0);
    fragPosition = worldPos.xyz;
    
    // Transform normal to world space (using normal matrix to handle non-uniform scaling)
    fragNormal = mat3(push.normalMatrix) * inNormal;
    
    // Pass through UV and color
    fragUV = inUV;
    fragColor = inColor;
    
    // Transform to clip space
    gl_Position = camera.viewProj * worldPos;
}

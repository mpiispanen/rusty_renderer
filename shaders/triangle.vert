#version 450

// Vertex input from vertex buffer
layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUV;
layout(location = 3) in vec4 inColor;

// Output to fragment shader
layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 fragUV;

void main() {
    // For 2D triangle, we only use X and Y from position
    gl_Position = vec4(inPosition.xy, 0.0, 1.0);
    fragColor = inColor.rgb;
    fragUV = inUV;
}

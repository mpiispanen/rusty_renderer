#version 450

// Input from vertex shader
layout(location = 0) in vec2 fragTexCoord;
layout(location = 1) in vec4 fragColor;

// Output color
layout(location = 0) out vec4 outColor;

// Texture and sampler bindings
layout(set = 0, binding = 0) uniform texture2D texSampler;
layout(set = 0, binding = 1) uniform sampler samp;

void main() {
    // Sample texture and multiply by vertex color
    vec4 texColor = texture(sampler2D(texSampler, samp), fragTexCoord);
    outColor = texColor * fragColor;
}

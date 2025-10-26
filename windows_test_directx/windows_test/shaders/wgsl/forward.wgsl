// Forward rendering shader for wgpu backend
// Matches GLSL forward shaders but in WGSL syntax

// Camera uniforms (set 0, binding 0)
struct CameraUniforms {
    view_proj: mat4x4<f32>,
};

// Lighting uniforms (set 0, binding 1)
struct Light {
    light_type: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
    
    dir_or_pos: vec3<f32>,
    _padding4: f32,
    
    color: vec3<f32>,
    intensity: f32,
};

struct LightingUniforms {
    ambient: vec3<f32>,
    light_count: u32,
    lights: array<Light, 8>,
};

// Transform uniforms (set 2, binding 0) - emulates push constants
struct TransformUniforms {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

// Material uniforms (set 0, binding 3)
struct MaterialUniforms {
    base_color: vec4<f32>,
    properties: vec4<f32>, // x = metallic, y = roughness, z = hasTexture, w = padding
};

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(0) @binding(1) var<uniform> lighting: LightingUniforms;
@group(0) @binding(2) var diffuse_texture: texture_2d<f32>;
@group(0) @binding(3) var<uniform> material: MaterialUniforms;
@group(0) @binding(4) var texture_sampler: sampler;
@group(1) @binding(0) var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

// Vertex shader
@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform position
    let world_pos = transform.model * vec4<f32>(in.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.position = camera.view_proj * world_pos;
    
    // Transform normal
    out.normal = normalize((transform.normal_matrix * vec4<f32>(in.normal, 0.0)).xyz);
    
    // Pass through
    out.uv = in.uv;
    out.color = in.color;
    
    return out;
}

// Fragment shader
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.normal);
    let V = normalize(-in.world_pos); // Camera at origin in view space
    
    // Get base color from material
    var base_color = material.base_color.rgb;
    
    // Sample texture if available
    if (material.properties.z > 0.5) {  // hasTexture flag
        let tex_color = textureSample(diffuse_texture, texture_sampler, in.uv);
        base_color = base_color * tex_color.rgb;  // Modulate with texture
    }
    
    // Blend with vertex color
    base_color = base_color * in.color.rgb;
    
    // Start with ambient
    var final_color = lighting.ambient * base_color;
    
    // Material properties (hardcoded for now)
    let shininess: f32 = 32.0;
    let specular_strength: f32 = 0.5;
    
    // Process each light
    for (var i: u32 = 0u; i < lighting.light_count; i = i + 1u) {
        let light = lighting.lights[i];
        
        var L: vec3<f32>;
        var attenuation: f32 = 1.0;
        
        if (light.light_type == 0u) {
            // Directional light
            L = normalize(-light.dir_or_pos);
        } else {
            // Point light
            let light_vec = light.dir_or_pos - in.world_pos;
            let distance = length(light_vec);
            L = normalize(light_vec);
            attenuation = 1.0 / (distance * distance);
        }
        
        // Diffuse
        let NdotL = max(dot(N, L), 0.0);
        let diffuse = NdotL * light.color * light.intensity;
        
        // Specular (Blinn-Phong)
        let H = normalize(L + V);
        let NdotH = max(dot(N, H), 0.0);
        let spec = pow(NdotH, shininess) * specular_strength;
        let specular = spec * light.color * light.intensity;
        
        // Accumulate
        final_color += (diffuse + specular) * base_color * attenuation;
    }
    
    return vec4<f32>(final_color, in.color.a);
}

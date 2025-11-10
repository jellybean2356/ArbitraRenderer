struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct ModelUniform {
    model: mat4x4<f32>,
    emissive: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};
@group(1) @binding(0)
var<uniform> model_uniform: ModelUniform;

struct Light {
    direction: vec3<f32>,
    _padding1: f32,
    color: vec3<f32>,
    _padding2: f32,
    intensity: f32,
    ambient_strength: f32,
    _padding3: f32,
    _padding4: f32,
};
@group(2) @binding(0)
var<uniform> light: Light;

const MAX_POINT_LIGHTS: u32 = 8u;

struct PointLight {
    position: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    _padding: f32,
};

struct PointLights {
    lights: array<PointLight, MAX_POINT_LIGHTS>,
    count: u32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};
@group(3) @binding(0)
var<uniform> point_lights: PointLights;

struct VertexInput {
    @location(0) position: vec3<f32>, 
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,  
    @location(0) color: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) uv: vec2<f32>,
}
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let world_position = model_uniform.model * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_position;
    output.world_position = world_position.xyz;

    let world_normal = (model_uniform.model * vec4<f32>(input.normal, 0.0)).xyz;
    output.world_normal = normalize(world_normal);

    output.color = input.color;
    output.uv = input.uv;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let ambient = light.ambient_strength * vec3<f32>(1.0, 1.0, 1.0);

    let light_dir = normalize(-light.direction);
    let diff = max(dot(input.world_normal, light_dir), 0.0);
    let directional_lighting = diff * light.color * light.intensity;

    var point_lighting = vec3<f32>(0.0, 0.0, 0.0);
    for (var i = 0u; i < point_lights.count; i = i + 1u) {
        let light_pos = point_lights.lights[i].position;
        let light_color = point_lights.lights[i].color;
        let light_intensity = point_lights.lights[i].intensity;
        
        let to_light = light_pos - input.world_position;
        let distance = length(to_light);
        let light_direction = normalize(to_light);
        
        let attenuation = light_intensity / max(distance * distance, 0.1);
        let diffuse_strength = max(dot(input.world_normal, light_direction), 0.0);
        
        point_lighting = point_lighting + (light_color * diffuse_strength * attenuation);
    }

    let lighting = ambient + directional_lighting + point_lighting;
    let lit_color = input.color * lighting;

    let final_color = lit_color + (input.color * model_uniform.emissive);

    return vec4<f32>(final_color, 1.0);
}
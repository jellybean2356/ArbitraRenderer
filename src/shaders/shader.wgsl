// camera view-projection matrix (shared across all objects)
struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

// per-instance model matrix (object transform)
struct ModelUniform {
    model: mat4x4<f32>,
};
@group(1) @binding(0)
var<uniform> model_uniform: ModelUniform;

struct VertexInput {
    @location(0) position: vec3<f32>, 
    @location(1) color: vec3<f32>,    
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,  
    @location(0) color: vec3<f32>,                
}
@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    // transform: model space -> world space -> clip space
    let world_position = model_uniform.model * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_position;

    output.color = input.color;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
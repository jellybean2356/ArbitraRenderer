use wgpu;
use bytemuck;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    // Front (white)
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0] },
    // Right (red)
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 0.0, 0.0] },
    // Back (green)
    Vertex { position: [ 0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [0.0, 1.0, 0.0] },
    // Left (blue)
    Vertex { position: [-0.5, -0.5, -0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [0.0, 0.0, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [0.0, 0.0, 1.0] },
    // Top (yellow)
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [1.0, 1.0, 0.0] },
    // Bottom (magenta)
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 0.0, 1.0] },
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 0.0, 1.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 0.0, 1.0] },
];

pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0,       // Front
    4, 5, 6, 6, 7, 4,       // Right
    8, 9, 10, 10, 11, 8,    // Back
    12, 13, 14, 14, 15, 12, // Left
    16, 17, 18, 18, 19, 16, // Top
    20, 21, 22, 22, 23, 20, // Bottom
];

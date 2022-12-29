use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, 1.0, -1.0],
        tex_coords: [0.0, 0.0],
    }, // B
    Vertex {
        position: [1.0, 1.0, -1.0],
        tex_coords: [1.0, 0.0],
    }, // A
    Vertex {
        position: [-1.0, -1.0, -1.0],
        tex_coords: [0.0, 1.0],
    }, // C
    Vertex {
        position: [1.0, -1.0, -1.0],
        tex_coords: [1.0, 1.0],
    }, // D
    Vertex {
        position: [-1.0, 1.0, 1.0],
        tex_coords: [1.0, 0.0],
    }, // H
    Vertex {
        position: [1.0, 1.0, 1.0],
        tex_coords: [0.0, 0.0],
    }, // E
    Vertex {
        position: [-1.0, -1.0, 1.0],
        tex_coords: [1.0, 1.0],
    }, // G
    Vertex {
        position: [1.0, -1.0, 1.0],
        tex_coords: [0.0, 1.0],
    }, // F
];

#[rustfmt::skip]
pub const INDICES: &[u16] = &[
    0, 1, 2, // Side 0
    2, 1, 3,
    4, 0, 6, // Side 1
    6, 0, 2,
    7, 5, 6, // Side 2
    6, 5, 4,
    3, 1, 7, // Side 3 
    7, 1, 5,
    4, 5, 0, // Side 4 
    0, 5, 1,
    3, 7, 2, // Side 5 
    2, 7, 6
];

impl Vertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        // https://sotrh.github.io/learn-wgpu/assets/img/vb_desc.63afb652.png
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct Vertex3 {
    pub position: [f32; 3],
    pub tex_coord: [f32; 2],
}

pub const SQUARE: [Vertex3; 4] = [
    Vertex3 {
        position: [0.5, 0.5, 0.0],
        tex_coord: [0.0, 0.0],
    },
    Vertex3 {
        position: [-0.5, 0.5, 0.0],
        tex_coord: [1.0, 0.0],
    },
    Vertex3 {
        position: [-0.5, -0.5, 0.0],
        tex_coord: [1.0, 1.0],
    },
    Vertex3 {
        position: [0.5, -0.5, 0.0],
        tex_coord: [0.0, 1.0],
    },
];

impl crate::buffer::GpuVertexBufferLayout for Vertex3 {
    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex3>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

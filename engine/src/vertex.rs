use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone, Debug)]
pub struct Vertex2 {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

pub const TRIANGLE: [Vertex2; 3] = [
    Vertex2 {
        position: [0.0, 0.5],
        color: [1.0, 0.0, 0.0],
    },
    Vertex2 {
        position: [-0.5, -0.5],
        color: [0.0, 1.0, 0.0],
    },
    Vertex2 {
        position: [0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    },
];

impl crate::buffer::GpuVertexBufferLayout for Vertex2 {
    fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex2>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x2,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x3,
                },
            ],
        }
    }
}

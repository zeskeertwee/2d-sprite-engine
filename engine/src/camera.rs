use crate::buffer::{GpuUniformBuffer, Uniform};
use cgmath::{Matrix4, SquareMatrix, Vector2};
use std::ops::Deref;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutEntry, BindingType,
    BufferBindingType, Device, Queue, ShaderStages,
};

pub struct Camera {
    uniform_buf: GpuUniformBuffer<CameraUniform>,
    bind_group: BindGroup,
    pub position: Vector2<f32>,
    pub height: f32,
    pub width: f32,
}

impl Camera {
    pub fn new(device: &Device, height: f32, width: f32) -> Self {
        let uniform = CameraUniform {
            view: Matrix4::identity().into(),
        };

        let buffer = GpuUniformBuffer::new(&device, &[uniform], Some("Camera UB"));
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &buffer.bind_group(&device, Some("Camera BGL")),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("Camera BG"),
        });

        Self {
            uniform_buf: buffer,
            bind_group,
            position: Vector2::new(0.0, 0.0),
            height,
            width,
        }
    }

    pub fn ortho_view_matrix(&self) -> cgmath::Matrix4<f32> {
        let half_height = self.height / 2.0;
        let top = self.position.y + half_height;
        let bottom = self.position.y - half_height;

        let half_width = self.width / 2.0;
        let right = self.position.x + half_width;
        let left = self.position.x - half_width;
        cgmath::ortho(left, right, bottom, top, 0.0, 1000.0)
    }

    pub fn update_uniform_buffer(&self, queue: &Queue) {
        let uniform = CameraUniform {
            view: self.ortho_view_matrix().into(),
        };

        self.uniform_buf.update(queue, &[uniform]);
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

impl Deref for Camera {
    type Target = GpuUniformBuffer<CameraUniform>;

    fn deref(&self) -> &Self::Target {
        &self.uniform_buf
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view: [[f32; 4]; 4],
}

impl Uniform for CameraUniform {
    fn bind_group_layout_entry() -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }
}

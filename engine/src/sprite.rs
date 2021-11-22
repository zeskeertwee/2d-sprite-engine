use crate::asset_management::GpuTextureRef;
use crate::buffer::GpuUniformBuffer;
use crate::texture::GpuTexture;
use cgmath::{Matrix4, Vector3};
use std::sync::Arc;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, Device, Queue, ShaderStages,
};

pub struct Sprite {
    pub(crate) texture: GpuTextureRef,
    /// the z component of the vector is used to draw the sprites on top of each other, in the correct order
    pub(crate) position: Vector3<f32>,
    pub(crate) scale: Vector3<f32>,
}

impl Sprite {
    pub fn new<T: Into<Vector3<f32>>>(texture: GpuTextureRef, position: T) -> Self {
        let pos = position.into();

        Self {
            texture,
            position: pos.into(),
            scale: Vector3::new(1.0, 0.5, 1.0),
        }
    }

    pub fn model_matrix(&self) -> Matrix4<f32> {
        let pos_mat = Matrix4::from_translation(self.position);
        let scale_mat = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        pos_mat * scale_mat
    }
}

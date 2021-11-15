use crate::asset_management::GpuTextureRef;
use crate::buffer::GpuUniformBuffer;
use crate::texture::GpuTexture;
use cgmath::Vector3;
use std::sync::Arc;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, Device, Queue, ShaderStages,
};

pub struct Sprite {
    pub(crate) texture: GpuTextureRef,
    /// the z component of the vector is used to draw the sprites on top of each other, in the correct order
    pub(crate) position: Vector3<f32>,
}

impl Sprite {
    pub fn new<T: Into<Vector3<f32>>>(texture: GpuTextureRef, position: T) -> Self {
        let pos = position.into();

        Self {
            texture,
            position: pos.into(),
        }
    }
}

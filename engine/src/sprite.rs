use crate::asset_management::GpuTextureRef;
use cgmath::Vector3;

pub struct Sprite {
    pub(crate) texture: GpuTextureRef,
    /// the z component of the vector is used to draw the sprites on top of each other, in the correct order
    pub(crate) position: Vector3<f32>,
}

impl Sprite {
    pub fn new<T: Into<Vector3<f32>>>(texture: GpuTextureRef, position: T) -> Self {
        Self {
            texture,
            position: position.into(),
        }
    }
}

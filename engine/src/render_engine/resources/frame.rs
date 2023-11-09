use wgpu::{SurfaceTexture, TextureView};

pub struct FrameResources {
    pub output: SurfaceTexture,
    pub view: TextureView,
}

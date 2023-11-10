use bevy_ecs::system::Resource;
use std::mem::swap;
use wgpu::{SurfaceTexture, TextureView};

#[derive(Resource)]
pub struct FrameResources {
    pub output: Option<SurfaceTexture>,
    pub view: TextureView,
}

impl FrameResources {
    pub fn output(&self) -> &SurfaceTexture {
        match &self.output {
            Some(v) => v,
            None => panic!("Attempt to get SurfaceTexture (was None)"),
        }
    }

    pub fn present(&mut self) {
        let mut old = None;
        swap(&mut old, &mut self.output);
        old.unwrap().present();
    }
}

use crate::asset_management::GpuTextureRef;
use crate::buffer::{GpuIndexBuffer, GpuVertexBuffer};
use crate::sprite::Sprite;
use crate::vertex::Vertex3;
use std::sync::Arc;
use wgpu::*;

pub struct SpriteRenderBatch<'a> {
    pass: RenderPass<'a>,
    pipeline: Arc<RenderPipeline>,
    vertex_buf: &'a GpuVertexBuffer<Vertex3>,
    index_buf: &'a GpuIndexBuffer<u32>,
    textures: Vec<Arc<GpuTextureRef>>,
    sprites: Vec<&'a Sprite>,
}

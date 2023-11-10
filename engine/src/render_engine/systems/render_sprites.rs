use crate::asset_management::ToUuid;
use crate::pipelines;
use crate::pipelines::sprite::SpritePushConstant;
use crate::render_engine::components::position::Position;
use crate::render_engine::components::texture::Texture;
use crate::render_engine::resources::{CommandBufferCollector, FrameResources};
use crate::render_engine::RenderEngineResources;
use bevy_ecs::prelude::*;
use parking_lot::Mutex;
use std::ops::Deref;
use wgpu::{
    Color, CommandBuffer, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, ShaderStages,
};

pub fn ecs_render_sprites(
    engine: Res<RenderEngineResources>,
    frame: Res<FrameResources>,
    command_collector: Res<CommandBufferCollector>,
    sprites: Query<(&Position, &Texture)>,
) {
    puffin::profile_function!();

    //let frame = match frame.deref() {
    //    Some(f) => f,
    //    None => panic!("FrameResources not initialized!"),
    //};

    let mut encoder = engine
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ecs_render_sprites_encoder"),
        });

    let pipeline = {
        puffin::profile_scope!("get_render_pipeline");
        engine
            .pipelines
            .get_render_pipeline(pipelines::sprite::SpriteRenderPipeline.uuid())
    };

    //let tex = engine.sprites.get(&0).unwrap().texture.load();

    let mut render_pass = {
        puffin::profile_scope!("begin_render_pass");
        let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[RenderPassColorAttachment {
                view: &frame.view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        rp.set_pipeline(&pipeline);
        rp.set_vertex_buffer(0, engine.sprite_square_vertex_buf.slice(..));
        rp.set_index_buffer(
            engine.sprite_square_index_buf.slice(..),
            engine.sprite_square_index_buf.index_format(),
        );
        rp.set_bind_group(0, &engine.camera.bind_group(), &[]);
        rp
    };

    {
        puffin::profile_scope!("draw_sprites");
        for (pos, tex) in sprites.iter() {
            let uniform =
                SpritePushConstant::new(crate::sprite::compute_model_matrix(pos.0), pos.0.z);

            //let model_mat = sprite.model_matrix();
            //let model: [u8; 64] =
            //    unsafe { std::mem::transmute(model_mat * cgmath::Matrix4::from_scale(200.0)) };
            //render_pass.set_push_constants(ShaderStages::VERTEX, 0, &model);
            //render_pass.set_push_constants(
            //    ShaderStages::VERTEX,
            //    std::mem::size_of_val(&model) as u32,
            //    &[sprite.position.z as u8, 0, 0, 0],
            //);
            render_pass.set_push_constants(ShaderStages::VERTEX, 0, &uniform.as_bytes());
            render_pass.set_bind_group(1, unsafe { tex.0.load().static_bind_group() }, &[]);
            render_pass.draw_indexed(0..engine.sprite_square_index_buf.data_count(), 0, 0..1);
        }
    }

    {
        puffin::profile_scope!("end_render_pass");
        drop(render_pass);
    }

    command_collector.push(encoder);
}

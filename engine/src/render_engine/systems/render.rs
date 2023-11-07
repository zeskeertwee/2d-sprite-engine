use crate::asset_management::ToUuid;
use crate::pipelines;
use crate::pipelines::sprite::SpritePushConstant;
use crate::render_engine::RenderEngineResources;
use bevy_ecs::prelude::*;
use std::ops::{Deref, DerefMut};
use std::time::Instant;
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, ShaderStages, TextureViewDescriptor,
};

pub fn render(mut engine: ResMut<RenderEngineResources>) {
    let idle_start = Instant::now();
    let output = engine.surface.get_current_texture().unwrap();
    engine.idle_time = idle_start.elapsed();
    let start = Instant::now();
    let view = output
        .texture
        .create_view(&TextureViewDescriptor::default());
    let mut encoder = engine
        .device
        .create_command_encoder(&CommandEncoderDescriptor { label: None });

    let pipeline = engine
        .pipelines
        .get_render_pipeline(pipelines::sprite::SpriteRenderPipeline.uuid());

    //let tex = self.sprites.get(&0).unwrap().texture.load();

    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[RenderPassColorAttachment {
            view: &view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: true,
            },
        }],
        depth_stencil_attachment: None,
    });

    render_pass.set_pipeline(&pipeline);
    render_pass.set_vertex_buffer(0, engine.sprite_square_vertex_buf.slice(..));
    render_pass.set_index_buffer(
        engine.sprite_square_index_buf.slice(..),
        engine.sprite_square_index_buf.index_format(),
    );
    render_pass.set_bind_group(0, &engine.camera.bind_group(), &[]);

    for (_, sprite) in engine.sprites.iter() {
        let uniform = SpritePushConstant::new(sprite.model_matrix(), sprite.position.z as u8);

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
        render_pass.set_bind_group(1, unsafe { sprite.texture.load().static_bind_group() }, &[]);
        render_pass.draw_indexed(0..engine.sprite_square_index_buf.data_count(), 0, 0..1);
    }
    drop(render_pass);

    let mut lock = engine.egui_integration.lock();
    lock.render(
        &engine.window,
        &mut encoder,
        &engine.device,
        &engine.queue,
        &view,
        &engine.config,
        engine.egui_debug_ui.write().deref_mut(),
    );
    drop(lock);

    engine.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    engine.last_frametime = start.elapsed();
}

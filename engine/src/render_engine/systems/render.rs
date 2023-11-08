use crate::asset_management::ToUuid;
use crate::pipelines;
use crate::pipelines::sprite::SpritePushConstant;
use crate::render_engine::RenderEngineResources;
use crate::scheduler::JobScheduler;
use crate::ui::integration::EguiRenderJob;
use bevy_ecs::prelude::*;
use log::{error, info};
use parking_lot::Mutex;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::Instant;
use wgpu::{
    Color, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment,
    RenderPassDescriptor, ShaderStages, SurfaceError, TextureViewDescriptor,
};

pub fn ecs_render(mut engine: ResMut<RenderEngineResources>) {
    puffin::profile_function!();
    let idle_start = Instant::now();
    let output = {
        puffin::profile_scope!("get_surface_texture");
        match engine.surface.get_current_texture() {
            Ok(v) => v,
            Err(SurfaceError::Lost) => {
                info!("Got SurfaceError::Lost, reconfiguring surface");
                engine.reconfigure_surface();
                return;
            }
            Err(SurfaceError::OutOfMemory) => {
                error!("Got SurfaceError::OutOfMemory");
                panic!("Got out of memory error!");
            }
            Err(e) => {
                error!("Got {}, skipping frame", e);
                return;
            }
        }
    };
    engine.idle_time = idle_start.elapsed();
    let start = Instant::now();
    let view = {
        puffin::profile_scope!("create_output_view");
        Arc::new(
            output
                .texture
                .create_view(&TextureViewDescriptor::default()),
        )
    };

    let (mut encoder, mut encoder2) = {
        puffin::profile_scope!("create_encoders");
        (
            engine
                .device
                .create_command_encoder(&CommandEncoderDescriptor { label: None }),
            Arc::new(Mutex::new(engine.device.create_command_encoder(
                &CommandEncoderDescriptor { label: None },
            ))),
        )
    };

    let egui_job_tracker = {
        puffin::profile_scope!("create_egui_job");
        let job = EguiRenderJob {
            encoder: Arc::clone(&encoder2),
            window: Arc::clone(&engine.window),
            output_view: Arc::clone(&view),
            surface_config: engine.config.clone(),
            app: Arc::clone(&engine.egui_debug_ui),
            integration: Arc::clone(&engine.egui_integration),
        };
        JobScheduler::submit(Box::new(job))
    };

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
        for (_, sprite) in engine.sprites.iter() {
            let uniform = SpritePushConstant::new(sprite.model_matrix(), sprite.position.z);

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
            render_pass.set_bind_group(
                1,
                unsafe { sprite.texture.load().static_bind_group() },
                &[],
            );
            render_pass.draw_indexed(0..engine.sprite_square_index_buf.data_count(), 0, 0..1);
        }
    }

    {
        puffin::profile_scope!("end_render_pass");
        drop(render_pass);
    }

    //engine.egui_integration.lock().render(
    //    &engine.window,
    //    &mut encoder,
    //    &engine.device,
    //    &engine.queue,
    //    &view,
    //    &engine.config,
    //    engine.egui_debug_ui.get_mut(),
    //);

    {
        puffin::profile_scope!("egui_job_flush");
        egui_job_tracker.flush().unwrap();
    }

    {
        puffin::profile_scope!("queue_submit_commands");
        engine.queue.submit([
            encoder.finish(),
            Arc::into_inner(encoder2).unwrap().into_inner().finish(),
        ]);
        output.present();
    }

    engine.last_frametime = start.elapsed();
}

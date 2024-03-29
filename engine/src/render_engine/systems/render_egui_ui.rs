use crate::render_engine::resources::{CommandBufferCollector, FrameResources};
use crate::render_engine::RenderEngineResources;
use bevy_ecs::system::Res;
use parking_lot::Mutex;
use std::ops::{Deref, DerefMut};
use wgpu::{CommandBuffer, CommandEncoderDescriptor};

pub fn ecs_render_egui_ui(
    engine: Res<RenderEngineResources>,
    frame: Res<FrameResources>,
    command_collector: Res<CommandBufferCollector>,
) {
    puffin::profile_function!();
    //let frame = match frame.deref() {
    //    Some(f) => f,
    //    None => panic!("FrameResources not initialized!"),
    //};

    let mut encoder = engine
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("ecs_render_egui_ui_encoder"),
        });

    engine.egui_integration.lock().render(
        &engine.window,
        &mut encoder,
        &engine.device,
        &engine.queue,
        &frame.view,
        &engine.config,
        engine.egui_debug_ui.write().deref_mut(),
    );

    command_collector.push(encoder);
}

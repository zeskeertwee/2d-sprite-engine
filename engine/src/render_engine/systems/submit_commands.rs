use crate::render_engine::resources::{CommandBufferCollector, FrameResources};
use crate::render_engine::RenderEngineResources;
use bevy_ecs::system::{Commands, Res, ResMut};
use log::error;
use parking_lot::Mutex;
use std::mem::swap;
use std::ops::DerefMut;
use wgpu::CommandBuffer;

pub fn ecs_render_submit_commands(
    mut commands: Commands,
    command_buffers: Res<CommandBufferCollector>,
    engine: Res<RenderEngineResources>,
    mut frame: ResMut<FrameResources>,
) {
    puffin::profile_function!();
    let collected_commands = command_buffers.take();
    engine.queue.submit(collected_commands);

    frame.present();
    commands.remove_resource::<FrameResources>();
}

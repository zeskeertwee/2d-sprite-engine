use crate::render_engine::resources::FrameResources;
use crate::render_engine::RenderEngineResources;
use bevy_ecs::system::{Res, ResMut};
use log::error;
use parking_lot::Mutex;
use std::mem::swap;
use std::ops::DerefMut;
use wgpu::CommandBuffer;

pub fn ecs_render_submit_commands(
    commands: Res<Mutex<Vec<CommandBuffer>>>,
    engine: Res<RenderEngineResources>,
    mut frame: ResMut<Option<FrameResources>>,
) {
    puffin::profile_function!();
    let mut lock = commands.lock();
    engine.queue.submit(lock.drain(..));
    if lock.len() > 0 {
        error!("Did not drain all commands!");
    }

    let mut old = None;
    swap(frame.deref_mut(), &mut old);
    let old = old.unwrap();
    old.output.present();
}

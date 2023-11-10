use bevy_ecs::prelude::World;
use bevy_ecs::system::Resource;
use bevy_ecs::world::FromWorld;
use parking_lot::Mutex;
use std::mem::swap;
use std::ops::DerefMut;
use wgpu::{CommandBuffer, CommandEncoder};

#[derive(Resource)]
pub struct CommandBufferCollector {
    inner: Mutex<Vec<CommandBuffer>>,
}

impl CommandBufferCollector {
    pub fn push(&self, encoder: CommandEncoder) {
        let command_buffer = encoder.finish();
        self.inner.lock().push(command_buffer);
    }

    pub fn take(&self) -> Vec<CommandBuffer> {
        let mut commands = Vec::new();
        let mut lock = self.inner.lock();
        swap(lock.deref_mut(), &mut commands);

        commands
    }
}

impl Default for CommandBufferCollector {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Vec::new()),
        }
    }
}

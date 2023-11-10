mod move_with_cursor;
pub use move_with_cursor::MoveWithCursor;

use crate::ecs::{EcsWorld, ScheduleStages};
use bevy_ecs::prelude::*;

pub fn initialize_in_world(world: &mut EcsWorld) {
    //world.schedule.add_system_to_stage(
    //    ScheduleStages::Update.to_str(),
    //    move_with_cursor::move_with_cursor_sys,
    //);
    world
        .schedule
        .add_systems(move_with_cursor::move_with_cursor_sys);
}

use crate::ecs::EcsWorld;
use bevy_ecs::prelude::*;
mod update_scripts;

pub fn initialize_systems_in_world(world: &mut EcsWorld) {
    //world.schedule.add_system_to_stage(ScheduleStages::PreUpdate.to_str(), update_scripts::update_scripts);
    world.schedule.add_systems(
        update_scripts::update_scripts.after(crate::ecs::systems::delta_time::update_delta_time),
    );

    world.world.insert_non_send_resource(super::create_lua_vm());
}

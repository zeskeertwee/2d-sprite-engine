use crate::ecs::resources::delta_time::LastDeltaTimeInstant;
use crate::ecs::resources::DeltaTime;
use bevy_ecs::system::ResMut;
use std::time::Instant;

pub fn update_delta_time(
    mut delta_time: ResMut<DeltaTime>,
    mut last_delta_time_instant: ResMut<LastDeltaTimeInstant>,
) {
    puffin::profile_function!();
    let elapsed = last_delta_time_instant.0.elapsed();
    delta_time.0 = elapsed;
    last_delta_time_instant.0 = Instant::now();
}

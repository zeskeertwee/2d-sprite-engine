use bevy_ecs::system::Resource;
use std::ops::Deref;
use std::time::{Duration, Instant};

#[derive(Resource)]
pub struct DeltaTime(pub Duration);

impl Deref for DeltaTime {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for DeltaTime {
    fn default() -> Self {
        Self(Duration::from_secs(0))
    }
}

#[derive(Resource)]
pub(crate) struct LastDeltaTimeInstant(pub Instant);

impl Default for LastDeltaTimeInstant {
    fn default() -> Self {
        Self(Instant::now())
    }
}

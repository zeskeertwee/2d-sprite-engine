use crate::asset_management::CacheCleanJob;
use crate::scheduler::JobScheduler;
use bevy_ecs::system::{Res, ResMut};
use log::trace;
use std::time::{Duration, Instant};

pub struct LastCacheClean(pub Instant);
pub struct CacheCleanInterval(pub Duration);

impl Default for LastCacheClean {
    fn default() -> Self {
        Self(Instant::now())
    }
}

impl Default for CacheCleanInterval {
    fn default() -> Self {
        Self(Duration::from_secs(10))
    }
}

pub fn clean_cache(mut last: ResMut<LastCacheClean>, interval: Res<CacheCleanInterval>) {
    puffin::profile_function!();
    if last.0.elapsed() > interval.0 {
        JobScheduler::submit(Box::new(CacheCleanJob));
        last.0 = Instant::now();
    }
}

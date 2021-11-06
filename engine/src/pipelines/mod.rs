mod job;
pub mod sprite;

use crate::asset_management::ToUuid;
use crate::scheduler::{Job, JobScheduler, JobState};
use ahash::{AHashMap, AHasher};
use lazy_static::lazy_static;
use log::{info, warn};
use parking_lot::Mutex;
use std::any::{type_name, Any};
use std::hash::Hasher;
use std::sync::Arc;
use uuid::Uuid;
use wgpu::*;
use winit::event::VirtualKeyCode::Mute;

lazy_static! {
    static ref RENDER_PIPELINES: [&'static dyn RenderPipelineInit; 1] =
        [&sprite::SpriteRenderPipeline];
}

pub trait RenderPipelineInit: ToUuid + Sync {
    fn init(&self, device: &Device, format: TextureFormat) -> anyhow::Result<RenderPipeline>;
}

pub struct Pipelines {
    render_pipelines: Arc<Mutex<AHashMap<Uuid, Arc<RenderPipeline>>>>,
}

impl Pipelines {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let mut res = Self {
            render_pipelines: Arc::new(Mutex::new(AHashMap::new())),
        };

        let mut job_trackers = Vec::new();

        for pipeline in RENDER_PIPELINES.iter() {
            let map = Arc::clone(&res.render_pipelines);
            let job = job::InitPipelineJob::new(*pipeline, format.clone(), map);
            job_trackers.push(JobScheduler::submit(Box::new(job)));
        }

        for tracker in job_trackers {
            tracker.flush();
        }

        res
    }

    /// This will panic if the id isn't present in the hashmap
    #[inline(always)]
    pub fn get_render_pipeline(&self, uuid: Uuid) -> Arc<RenderPipeline> {
        let mut lock = self.render_pipelines.lock();
        Arc::clone(lock.get(&uuid).expect(&format!("Render pipeline with asset UUID {} isn't initialized yet, or the asset UUID is invalid.", uuid)))
    }
}

fn hash_type_name<T: Any>() -> u64 {
    let mut hasher = AHasher::default();
    hasher.write(type_name::<T>().as_bytes());
    hasher.finish()
}

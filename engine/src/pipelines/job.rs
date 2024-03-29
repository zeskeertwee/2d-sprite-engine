use crate::asset_management::{ToUuid, Uuid};
use crate::pipelines::RenderPipelineInit;
use crate::scheduler::{Job, JobFrequency};
use ahash::AHashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use wgpu::{Device, Queue, RenderPipeline, TextureFormat};

pub struct InitPipelineJob {
    format: TextureFormat,
    pipeline: &'static dyn RenderPipelineInit,
    map: Arc<Mutex<AHashMap<Uuid, Arc<RenderPipeline>>>>,
}

impl ToUuid for InitPipelineJob {}

impl Job for InitPipelineJob {
    fn get_freq(&self) -> JobFrequency {
        JobFrequency::Once
    }

    fn run(&mut self, device: &Device, _: &Queue) -> anyhow::Result<()> {
        let pipeline = Arc::new(self.pipeline.init(device, self.format)?);
        self.map.lock().insert(self.pipeline.uuid(), pipeline);
        Ok(())
    }
}

impl InitPipelineJob {
    pub fn new(
        pipeline: &'static dyn RenderPipelineInit,
        format: TextureFormat,
        map: Arc<Mutex<AHashMap<Uuid, Arc<RenderPipeline>>>>,
    ) -> Self {
        Self {
            pipeline,
            format,
            map,
        }
    }
}

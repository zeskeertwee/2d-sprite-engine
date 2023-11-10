use crate::asset_management::{AssetLoader, ToUuid};
use crate::scheduler::{Job, JobFrequency};
use anyhow::Result;
use wgpu::{Device, Queue};

pub struct CacheCleanJob;

impl ToUuid for CacheCleanJob {}

impl Job for CacheCleanJob {
    fn get_freq(&self) -> JobFrequency {
        JobFrequency::Periodically
    }

    fn run(&mut self, _: &Device, _: &Queue) -> Result<()> {
        AssetLoader::clean_cache();

        Ok(())
    }
}

use crate::asset_management::{AssetLoader, GpuTextureRef, ToUuid, Uuid, UUID_NAMESPACE_ASSETS};
use crate::render_engine::texture::GpuTexture;
use crate::scheduler::{Job, JobFrequency};
use anyhow::Result;
use image::ImageFormat;
use std::sync::Arc;
use wgpu::{Device, Queue};

pub struct TextureLoadJob {
    pub id: String,
    pub tex: GpuTextureRef,
    pub format: Option<ImageFormat>,
}

impl ToUuid for TextureLoadJob {}

impl Job for TextureLoadJob {
    fn get_freq(&self) -> JobFrequency {
        JobFrequency::Once
    }

    fn run(&mut self, device: &Device, queue: &Queue) -> Result<()> {
        let data = AssetLoader::get_asset_uncached(&self.id)?;
        let uuid = Uuid::new_v5(&UUID_NAMESPACE_ASSETS, self.id.as_bytes());
        let texture = match self.format {
            Some(format) => GpuTexture::new_from_data_with_format(
                device,
                queue,
                &data,
                format,
                Some(&self.id),
                uuid,
            ),
            None => GpuTexture::new_from_data(device, queue, &data, Some(&self.id), uuid),
        }?;

        let cached_texture = Arc::new(texture);
        AssetLoader::insert_into_texture_cache(&self.id, cached_texture);
        self.tex.swap(Arc::new(uuid));
        Ok(())
    }
}

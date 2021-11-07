use anyhow::Result;
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::num::NonZeroU32;
use std::sync::Arc;
use wgpu::*;

pub struct GpuTexture {
    inner: Texture,
    pub(crate) bind_group: Arc<BindGroup>,
}

impl GpuTexture {
    pub fn new_from_data(
        device: &Device,
        queue: &Queue,
        data: &[u8],
        label: Option<&str>,
    ) -> Result<Self> {
        let image = image::load_from_memory(data)?;
        Ok(Self::new_from_image(
            device,
            queue,
            &image,
            label.unwrap_or("unnamed"),
        ))
    }

    pub fn new_from_data_with_format(
        device: &Device,
        queue: &Queue,
        data: &[u8],
        format: ImageFormat,
        label: Option<&str>,
    ) -> Result<Self> {
        let image = image::load_from_memory_with_format(data, format)?;
        Ok(Self::new_from_image(
            device,
            queue,
            &image,
            label.unwrap_or("unnamed"),
        ))
    }

    pub fn new_from_image(
        device: &Device,
        queue: &Queue,
        image: &DynamicImage,
        label: &str,
    ) -> Self {
        let texture_size = Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            label: Some(&format!("{} TEX", label)),
        });

        let img_data = image.to_rgba8().into_raw();

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &img_data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * image.width()),
                rows_per_image: NonZeroU32::new(image.height()),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout = Self::build_bind_group_layout(&device, label);

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some(&format!("{} BG", label)),
        });

        Self {
            inner: texture,
            bind_group: Arc::new(texture_bind_group),
        }
    }

    pub fn build_bind_group_layout(device: &Device, label: &str) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
            label: Some(&format!("{} BGL", label)),
        })
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }
}

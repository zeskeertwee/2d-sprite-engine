use super::pipelines;
use ahash::AHashMap;
use image::ImageFormat;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::asset_management::GpuTextureRef;
use crate::asset_management::{AssetLoader, ToUuid};
use crate::buffer::{GpuIndexBuffer, GpuVertexBuffer};
use crate::camera::Camera;
use crate::pipelines::Pipelines;
use crate::scheduler::JobScheduler;
use crate::sprite::Sprite;
use crate::texture::GpuTexture;
use crate::vertex::Vertex2;
use log::warn;
use pollster::block_on;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct RenderEngine {
    surface: Surface,
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    pipelines: Pipelines,
    camera: Camera,
    sprite_square_vertex_buf: GpuVertexBuffer<Vertex2>,
    sprite_square_index_buf: GpuIndexBuffer<u16>,
    sprites: AHashMap<u64, Sprite>,
    sprite_id_counter: u64,
    last_frametime: Duration,
    idle_time: Duration,
}

impl RenderEngine {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // should make it work on linux, macos and windows
        // since vulkan works on linux and windows, and metal on macos
        let instance = Instance::new(Backends::VULKAN | Backends::METAL);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(),
                limits: Limits::default(),
            },
            None,
        ))
        .unwrap();

        let (device, queue) = (Arc::new(device), Arc::new(queue));

        JobScheduler::init_device_queue(Arc::clone(&device), Arc::clone(&queue));
        // TODO: use propper amount of CPU cores
        JobScheduler::spawn_workers(15).unwrap();

        AssetLoader::set_tex_placeholder(&device, &queue, "placeholder-32.png", ImageFormat::Png)
            .unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let pipelines = Pipelines::new(config.format);

        let sprite_vertex_buf =
            GpuVertexBuffer::new(&device, &crate::vertex::SQUARE, Some("Square VB"));
        let sprite_index_buf = GpuIndexBuffer::new(&device, &[0, 1, 2, 0, 2, 3], Some("Square IB"));
        let camera = Camera::new(&device, size.height as f32, size.width as f32);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            pipelines,
            camera,
            sprite_square_vertex_buf: sprite_vertex_buf,
            sprite_square_index_buf: sprite_index_buf,
            sprite_id_counter: 0,
            sprites: AHashMap::new(),
            last_frametime: Duration::new(0, 0),
            idle_time: Duration::new(0, 0),
        }
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.reconfigure_surface();
        } else {
            warn!("Attempt to resize window to a size where x = 0 or where y = 0");
        }
    }

    pub fn reconfigure_surface(&self) {
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update(&mut self) {
        self.camera.update_uniform_buffer(&self.queue);
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let idle_start = Instant::now();
        let output = self.surface.get_current_texture()?;
        self.idle_time = idle_start.elapsed();
        let start = Instant::now();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let pipeline = self
            .pipelines
            .get_render_pipeline(pipelines::sprite::SpriteRenderPipeline.uuid());

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        let mut sprite_resources = WgpuRenderPass {
            pass: render_pass,
            uniform_bind_group: &self.camera,
            bind_groups: Vec::new(),
            vertex_buf: &self.sprite_square_vertex_buf,
            index_buf: &self.sprite_square_index_buf,
            pipeline: &*self
                .pipelines
                .get_render_pipeline(pipelines::sprite::SpriteRenderPipeline.uuid()),
        };

        for sprite in self.sprites.values() {
            let bind_group = match &sprite.texture {
                GpuTextureRef::Swappable(x) => {
                    let tex = x.load();
                    Arc::clone(&tex.bind_group)
                }
                GpuTextureRef::Shared(x) => Arc::clone(&x.bind_group),
            };

            sprite_resources.bind_groups.push(bind_group);
        }

        //sprite_resources.pass.set_pipeline(&pipeline);
        //sprite_resources
        //    .pass
        //    .set_vertex_buffer(0, self.sprite_square_vertex_buf.slice(..));
        //sprite_resources.pass.set_index_buffer(
        //    self.sprite_square_index_buf.slice(..),
        //    self.sprite_square_index_buf.index_format(),
        //);
        //
        //for bind_group in sprite_resources.bind_groups.iter() {
        //    sprite_resources.pass.set_bind_group(0, &bind_group, &[]);
        //    sprite_resources.pass.draw_indexed(
        //        0..self.sprite_square_index_buf.data_count(),
        //        0,
        //        0..1,
        //    );
        //}
        //
        //drop(sprite_resources);

        sprite_resources.submit();

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.last_frametime = start.elapsed();
        Ok(())
    }

    pub fn frametime(&self) -> &Duration {
        &self.last_frametime
    }

    pub fn idle_time(&self) -> &Duration {
        &self.idle_time
    }

    /// 1.0 means the gpu is spending no time idle, while 0.0 means the gpu is not spending any time rendering
    pub fn gpu_busy_ratio(&self) -> f64 {
        let total_time = self.idle_time.as_secs_f64() + self.last_frametime.as_secs_f64();
        self.last_frametime.as_secs_f64() / total_time
    }

    pub fn insert_sprite(&mut self, sprite: Sprite) -> u64 {
        let id = self.sprite_id_counter;
        self.sprite_id_counter += 1;
        self.sprites.insert(id, sprite);
        id
    }
}

struct WgpuRenderPass<'a> {
    pass: RenderPass<'a>,
    pipeline: &'a RenderPipeline,
    bind_groups: Vec<Arc<BindGroup>>,
    uniform_bind_group: &'a Camera,
    vertex_buf: &'a GpuVertexBuffer<Vertex2>,
    index_buf: &'a GpuIndexBuffer<u16>,
}

impl<'a> WgpuRenderPass<'a> {
    pub fn submit(mut self) {
        self.pass.set_pipeline(self.pipeline);
        self.pass
            .set_bind_group(0, &self.uniform_bind_group.bind_group(), &[]);
        self.pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        self.pass
            .set_index_buffer(self.index_buf.slice(..), self.index_buf.index_format());

        for sprite in self.bind_groups {
            // SAFETY: trust me bro
            let borrow: &'static Arc<BindGroup> = unsafe { std::mem::transmute(&sprite) };
            self.pass.set_bind_group(1, &borrow, &[]);
            self.pass
                .draw_indexed(0..self.index_buf.data_count(), 0, 0..1);
        }
    }
}

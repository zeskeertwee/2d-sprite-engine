mod ecs;
mod systems;

use super::pipelines;
use ahash::AHashMap;
use image::ImageFormat;
use std::cell::RefCell;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::asset_management::{AssetLoader, ToUuid};
use crate::buffer::{GpuIndexBuffer, GpuVertexBuffer};
use crate::camera::Camera;
use crate::pipelines::sprite::SpritePushConstant;
use crate::pipelines::Pipelines;
use crate::scheduler::JobScheduler;
use crate::sprite::Sprite;
use crate::ui::integration::{EguiIntegration, EguiRequestRedrawEvent};
use crate::ui::DebugUi;
use crate::vertex::Vertex2;
use log::{info, warn};
use parking_lot::{Mutex, RwLock};
use pollster::block_on;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};

const PUSH_CONSTANT_SIZE_LIMIT: u32 = 256;

pub struct RenderEngineResources {
    window: Window,
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
    egui_integration: Mutex<EguiIntegration>,
    egui_debug_ui: RwLock<DebugUi>,
}

impl RenderEngineResources {
    pub fn new(window: Window, event_loop: &EventLoop<EguiRequestRedrawEvent>) -> Self {
        let size = window.inner_size();

        // should make it work on linux, macos and windows
        // since vulkan works on linux and windows, and metal on macos
        let instance = Instance::new(Backends::VULKAN | Backends::METAL);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let mut limits = Limits::default();
        limits.max_push_constant_size = PUSH_CONSTANT_SIZE_LIMIT;

        let mut features = Features::empty();
        features.set(Features::PUSH_CONSTANTS, true);

        let (device, queue) = block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                features,
                limits,
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

        let egui = EguiIntegration::new(&device, event_loop, &config, window.scale_factor() as f32);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            pipelines,
            camera,
            egui_integration: Mutex::new(egui),
            egui_debug_ui: RwLock::new(DebugUi::default()),
            sprite_square_vertex_buf: sprite_vertex_buf,
            sprite_square_index_buf: sprite_index_buf,
            sprite_id_counter: 0,
            sprites: AHashMap::new(),
            last_frametime: Duration::new(0, 0),
            idle_time: Duration::new(0, 0),
        }
    }

    pub fn process_event(&mut self, event: &Event<EguiRequestRedrawEvent>) {
        self.egui_integration.lock().process_event(event);
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.reconfigure_surface();
            self.camera.window_resize(new_size);
        } else {
            warn!("Attempt to resize window to a size where x = 0 or where y = 0");
        }
    }

    pub fn update_scale_factor(&mut self, scale_factor: f64) {
        self.egui_integration
            .lock()
            .set_scale_factor(scale_factor as f32);
    }

    pub fn reconfigure_surface(&self) {
        puffin::profile_function!();
        self.surface.configure(&self.device, &self.config);
    }

    pub fn update(&mut self) {
        puffin::profile_function!();
        self.camera.update_uniform_buffer(&self.queue);
        let frametime = self.total_frame_time().as_secs_f64();
        self.egui_debug_ui
            .get_mut()
            .fps_window_mut()
            .set_frametime(frametime);

        if self.config.present_mode != self.egui_debug_ui.read().fps_window().present_mode() {
            info!(
                "Updating present mode from {:?} to {:?} (reason: DebugUi)",
                self.config.present_mode,
                self.egui_debug_ui.read().fps_window().present_mode()
            );
            self.config.present_mode = self.egui_debug_ui.read().fps_window().present_mode();
            self.reconfigure_surface();
        }

        self.egui_debug_ui
            .get_mut()
            .cache_window_mut()
            .update(&self.device, self.egui_integration.lock().render_pass_mut());
    }

    pub fn render(&mut self) -> Result<(), SurfaceError> {
        puffin::profile_function!();
        let idle_start = Instant::now();
        let output = {
            puffin::profile_scope!("get_surface_texture");
            self.surface.get_current_texture()?
        };
        self.idle_time = idle_start.elapsed();
        let start = Instant::now();
        let view = {
            puffin::profile_scope!("create_output_view");
            output
                .texture
                .create_view(&TextureViewDescriptor::default())
        };
        let mut encoder = {
            puffin::profile_scope!("create_encoder");
            self.device
                .create_command_encoder(&CommandEncoderDescriptor { label: None })
        };

        let pipeline = {
            puffin::profile_scope!("get_render_pipeline");
            self.pipelines
                .get_render_pipeline(pipelines::sprite::SpriteRenderPipeline.uuid())
        };

        //let tex = self.sprites.get(&0).unwrap().texture.load();

        let mut render_pass = {
            puffin::profile_scope!("begin_render_pass");
            let mut rp = encoder.begin_render_pass(&RenderPassDescriptor {
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

            rp.set_pipeline(&pipeline);
            rp.set_vertex_buffer(0, self.sprite_square_vertex_buf.slice(..));
            rp.set_index_buffer(
                self.sprite_square_index_buf.slice(..),
                self.sprite_square_index_buf.index_format(),
            );
            rp.set_bind_group(0, &self.camera.bind_group(), &[]);
            rp
        };

        {
            puffin::profile_scope!("update_sprite_constants");
            for (_, sprite) in self.sprites.iter() {
                let uniform =
                    SpritePushConstant::new(sprite.model_matrix(), sprite.position.z as u8);

                //let model_mat = sprite.model_matrix();
                //let model: [u8; 64] =
                //    unsafe { std::mem::transmute(model_mat * cgmath::Matrix4::from_scale(200.0)) };
                //render_pass.set_push_constants(ShaderStages::VERTEX, 0, &model);
                //render_pass.set_push_constants(
                //    ShaderStages::VERTEX,
                //    std::mem::size_of_val(&model) as u32,
                //    &[sprite.position.z as u8, 0, 0, 0],
                //);
                render_pass.set_push_constants(ShaderStages::VERTEX, 0, &uniform.as_bytes());
                render_pass.set_bind_group(
                    1,
                    unsafe { sprite.texture.load().static_bind_group() },
                    &[],
                );
                render_pass.draw_indexed(0..self.sprite_square_index_buf.data_count(), 0, 0..1);
            }
        }
        drop(render_pass);

        self.egui_integration.lock().render(
            &self.window,
            &mut encoder,
            &self.device,
            &self.queue,
            &view,
            &self.config,
            self.egui_debug_ui.get_mut(),
        );

        {
            puffin::profile_scope!("queue_submit_commands");
            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        self.last_frametime = start.elapsed();
        Ok(())
    }

    pub fn frametime(&self) -> &Duration {
        &self.last_frametime
    }

    pub fn idle_time(&self) -> &Duration {
        &self.idle_time
    }

    pub fn total_frame_time(&self) -> Duration {
        self.idle_time + self.last_frametime
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

    pub fn get_sprite_mut(&mut self, id: u64) -> Option<&mut Sprite> {
        self.sprites.get_mut(&id)
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }

    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn window_id(&self) -> WindowId {
        self.window.id()
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

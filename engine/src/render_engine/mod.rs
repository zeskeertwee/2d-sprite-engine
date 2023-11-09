pub mod components;
pub(crate) mod ecs;
mod resources;
mod systems;

use std::ops::DerefMut;
use image::ImageFormat;
use std::sync::Arc;
use std::time::Duration;

use crate::asset_management::AssetLoader;
use crate::buffer::{GpuIndexBuffer, GpuVertexBuffer};
use crate::camera::Camera;
use crate::pipelines::Pipelines;
use crate::scheduler::JobScheduler;
use crate::ui::integration::{EguiIntegration, EguiRequestRedrawEvent};
use crate::ui::DebugUi;
use crate::vertex::Vertex2;
use log::warn;
use parking_lot::{Mutex, RwLock};
use pollster::block_on;
use wgpu::*;
use winit::dpi::PhysicalSize;
use winit::event::Event;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowId};

const PUSH_CONSTANT_SIZE_LIMIT: u32 = 256;

pub struct RenderEngineResources {
    window: Arc<Window>,
    surface: Surface,
    device: Arc<Device>,
    queue: Arc<Queue>,
    config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    pipelines: Pipelines,
    camera: Camera,
    sprite_square_vertex_buf: GpuVertexBuffer<Vertex2>,
    sprite_square_index_buf: GpuIndexBuffer<u16>,
    last_frametime: Duration,
    idle_time: Duration,
    egui_integration: Arc<Mutex<EguiIntegration>>,
    egui_debug_ui: Arc<RwLock<DebugUi>>,
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
            window: Arc::new(window),
            surface,
            device,
            queue,
            config,
            size,
            pipelines,
            camera,
            egui_integration: Arc::new(Mutex::new(egui)),
            egui_debug_ui: Arc::new(RwLock::new(DebugUi::default())),
            sprite_square_vertex_buf: sprite_vertex_buf,
            sprite_square_index_buf: sprite_index_buf,
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
    
    pub fn with_debug_ui<F: FnOnce(&mut DebugUi) -> R, R>(&self, f: F) -> R {
        (f)(self.egui_debug_ui.write().deref_mut())
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

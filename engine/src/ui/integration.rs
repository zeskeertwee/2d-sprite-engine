use egui::FontDefinitions;
use egui_wgpu_backend;
use egui_wgpu_backend::ScreenDescriptor;
use egui_winit_platform;
use egui_winit_platform::{Platform, PlatformDescriptor};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Instant;
use wgpu::{CommandEncoder, Device, Queue, SurfaceConfiguration, TextureView};
use winit::event::Event;
use winit::event_loop::{EventLoop, EventLoopProxy};
use winit::window::Window;

pub enum EguiRequestRedrawEvent {
    RequestRedraw,
}

struct RepaintSignaller(Mutex<EventLoopProxy<EguiRequestRedrawEvent>>);

impl epi::backend::RepaintSignal for RepaintSignaller {
    fn request_repaint(&self) {
        self.0
            .lock()
            .send_event(EguiRequestRedrawEvent::RequestRedraw);
    }
}

pub struct EguiIntegration {
    platform: Platform,
    render_pass: egui_wgpu_backend::RenderPass,
    repaint_signaller: Arc<RepaintSignaller>,
    previous_frame_time: Option<f32>,
    scale_factor: f32,
    start_time: Instant,
}

impl EguiIntegration {
    pub fn new(
        device: &Device,
        event_loop: &EventLoop<EguiRequestRedrawEvent>,
        surface_config: &SurfaceConfiguration,
        scale_factor: f32,
    ) -> Self {
        let repaint_signaller = Arc::new(RepaintSignaller(Mutex::new(event_loop.create_proxy())));

        let platform = Platform::new(PlatformDescriptor {
            physical_width: surface_config.width,
            physical_height: surface_config.height,
            scale_factor: scale_factor as f64,
            font_definitions: FontDefinitions::default(),
            style: Default::default(),
        });

        let render_pass = egui_wgpu_backend::RenderPass::new(device, surface_config.format, 1);

        Self {
            render_pass,
            platform,
            repaint_signaller,
            previous_frame_time: None,
            scale_factor,
            start_time: Instant::now(),
        }
    }

    pub fn process_event(&mut self, event: &Event<EguiRequestRedrawEvent>) {
        self.platform.handle_event(event);
    }

    pub fn render(
        &mut self,
        window: &Window,
        encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        output_view: &TextureView,
        surface_config: &SurfaceConfiguration,
        app: &mut dyn epi::App,
    ) {
        puffin::profile_function!("egui_render");
        self.platform
            .update_time(self.start_time.elapsed().as_secs_f64());
        let render_start = Instant::now();
        self.platform.begin_frame();
        let app_output = epi::backend::AppOutput::default();

        let mut frame = epi::Frame(std::sync::Arc::new(std::sync::Mutex::new(
            epi::backend::FrameData {
                info: epi::IntegrationInfo {
                    web_info: None,
                    cpu_usage: self.previous_frame_time,
                    native_pixels_per_point: Some(self.scale_factor),
                    name: "egui-sprite-engine",
                    prefer_dark_mode: Some(true),
                },
                output: app_output,
                repaint_signal: self.repaint_signaller.clone(),
            },
        )));

        app.update(&self.platform.context(), &mut frame);

        let (_output, paint_commands) = self.platform.end_frame(Some(window));
        let paint_jobs = self.platform.context().tessellate(paint_commands);
        self.previous_frame_time = Some(render_start.elapsed().as_secs_f64() as f32);

        let screen_descriptor = ScreenDescriptor {
            physical_width: surface_config.width,
            physical_height: surface_config.height,
            scale_factor: window.scale_factor() as f32,
        };

        self.render_pass
            .update_texture(device, queue, &self.platform.context().font_image());
        self.render_pass.update_user_textures(device, queue);
        self.render_pass
            .update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

        self.render_pass
            .execute(encoder, output_view, &paint_jobs, &screen_descriptor, None)
            .expect("successful egui render pass");
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.scale_factor = scale_factor;
    }

    pub fn render_pass_mut(&mut self) -> &mut egui_wgpu_backend::RenderPass {
        &mut self.render_pass
    }
}

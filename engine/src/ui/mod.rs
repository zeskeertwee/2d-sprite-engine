use crate::ui::cache::CacheDebugUi;
use crate::ui::fps::DebugFrametimeWindow;
use egui::{CtxRef, Ui, Vec2};
use egui_wgpu_backend::epi::Frame;
use wgpu::PresentMode;

mod cache;
mod fps;
pub mod integration;

trait EguiWindow {
    fn title(&self) -> &'static str;
    fn draw(&mut self, ui: &mut Ui);
}

#[derive(Default)]
pub struct DebugUi {
    fps_window: DebugFrametimeWindow,
    cache_window: CacheDebugUi,
}

impl epi::App for DebugUi {
    fn name(&self) -> &str {
        "Debug UI for sprite-engine"
    }

    fn update(&mut self, ctx: &CtxRef, _frame: &mut Frame<'_>) {
        egui::Window::new(self.fps_window.title()).show(ctx, |ui| {
            self.fps_window.draw(ui);
        });

        egui::Window::new(self.cache_window.title()).show(ctx, |ui| {
            self.cache_window.draw(ui);
        });
    }
}

impl DebugUi {
    pub fn fps_window_mut(&mut self) -> &mut DebugFrametimeWindow {
        &mut self.fps_window
    }

    pub fn fps_window(&self) -> &DebugFrametimeWindow {
        &self.fps_window
    }

    pub fn cache_window_mut(&mut self) -> &mut CacheDebugUi {
        &mut self.cache_window
    }
}

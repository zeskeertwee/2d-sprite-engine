use crate::ui::cache::CacheDebugUi;
use crate::ui::fps::DebugFrametimeWindow;
use crate::ui::scheduler::SchedulerWorkerThreadWindow;
use ahash::AHashMap;
use egui::{CtxRef, Ui, Vec2};
use epi::Frame;
use std::collections::HashMap;
use wgpu::PresentMode;

mod cache;
mod fps;
pub mod integration;
mod scheduler;

trait EguiWindow {
    fn title(&self) -> &'static str;
    fn draw(&mut self, ui: &mut Ui);
    fn menu_category(&self) -> MenuCategory;
}

pub enum MenuCategory {
    Debug,
    Performance,
}

#[derive(Default)]
pub struct DebugUi {
    show_fps_window: bool,
    fps_window: DebugFrametimeWindow,
    show_cache_window: bool,
    cache_window: CacheDebugUi,
    show_scheduler_window: bool,
    scheduler_window: SchedulerWorkerThreadWindow,
}

impl epi::App for DebugUi {
    fn name(&self) -> &str {
        "Debug UI for sprite-engine"
    }

    fn update(&mut self, ctx: &CtxRef, _frame: &Frame) {
        egui::TopBottomPanel::top("top_menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.monospace(format!("FPS: {:.2}", 1.0 / self.fps_window.frametime));
                ui.add_space(10.0);

                ui.menu_button("Debug", |ui| {
                    ui.checkbox(&mut self.show_cache_window, "Texture Cache");
                    ui.checkbox(&mut self.show_scheduler_window, "Scheduler");
                });

                ui.menu_button("Preformance", |ui| {
                    ui.checkbox(&mut self.show_fps_window, "FPS & Present mode");
                });
            });
        });

        if self.show_fps_window {
            egui::Window::new(self.fps_window.title()).show(ctx, |ui| {
                self.fps_window.draw(ui);
            });
        }

        if self.show_cache_window {
            egui::Window::new(self.cache_window.title()).show(ctx, |ui| {
                self.cache_window.draw(ui);
            });
        }

        if self.show_scheduler_window {
            egui::Window::new(self.scheduler_window.title()).show(ctx, |ui| {
                self.scheduler_window.draw(ui);
            });
        }
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

use super::{EguiWindow, MenuCategory};
use egui::Ui;
use puffin_egui;

pub struct PuffinProfilerWindow;

impl Default for PuffinProfilerWindow {
    fn default() -> Self {
        Self {}
    }
}

impl EguiWindow for PuffinProfilerWindow {
    fn title(&self) -> &'static str {
        "Puffin profiler"
    }

    fn menu_category(&self) -> MenuCategory {
        MenuCategory::Performance
    }

    fn draw(&mut self, ui: &mut Ui) {
        puffin_egui::profiler_ui(ui)
    }
}

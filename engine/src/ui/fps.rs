use super::EguiWindow;
use crate::ui::MenuCategory;
use egui::Ui;
use std::collections::VecDeque;
use wgpu::PresentMode;

const MAX_FRAMETIME_SAMPLES: usize = 60;

pub struct DebugFrametimeWindow {
    /// the frametime, in seconds
    avg_frametime: f64,
    frametimes: VecDeque<f64>,
    present_mode: PresentMode,
}

impl EguiWindow for DebugFrametimeWindow {
    fn title(&self) -> &'static str {
        "Frametime"
    }

    fn menu_category(&self) -> MenuCategory {
        MenuCategory::Performance
    }

    fn draw(&mut self, ui: &mut Ui) {
        puffin::profile_function!("DebugFrametimeWindow");
        ui.label(format!(
            "Frametime: {:.2}ms {}",
            self.avg_frametime * 1000.0,
            if self.present_mode == PresentMode::Mailbox {
                "POSSIBLY INACCURATE"
            } else {
                ""
            }
        ));
        ui.add_space(15.0);
        ui.label(format!(
            "FPS: {:.2} {}",
            1.0 / self.avg_frametime,
            if self.present_mode == PresentMode::Mailbox {
                "POSSIBLY INACCURATE"
            } else {
                ""
            }
        ));
        ui.add_space(15.0);
        egui::ComboBox::from_label("Select a wgpu present mode")
            .selected_text(format!("{:?}", self.present_mode))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.present_mode, PresentMode::Fifo, "Fifo");
                ui.selectable_value(&mut self.present_mode, PresentMode::Mailbox, "Mailbox");
                ui.selectable_value(&mut self.present_mode, PresentMode::Immediate, "Immediate");
            });
        ui.label(present_mode_description(self.present_mode));
    }
}

impl Default for DebugFrametimeWindow {
    fn default() -> Self {
        Self {
            avg_frametime: 0.0,
            frametimes: VecDeque::new(),
            present_mode: PresentMode::Fifo,
        }
    }
}

impl DebugFrametimeWindow {
    pub fn set_frametime(&mut self, frametime: f64) {
        self.frametimes.push_back(frametime);
        if self.frametimes.len() > MAX_FRAMETIME_SAMPLES {
            self.frametimes.pop_front();
        }

        self.avg_frametime = self.frametimes.iter().sum::<f64>() / self.frametimes.len() as f64;
    }

    pub fn get_avg_frametime(&self) -> f64 {
        self.avg_frametime
    }

    pub fn present_mode(&self) -> PresentMode {
        self.present_mode
    }
}

fn present_mode_description(mode: PresentMode) -> &'static str {
    match mode {
        PresentMode::Fifo => "Waits for VBlank before presenting, caps the framerate to the refresh rate, no tearing should occur",
        PresentMode::Mailbox => "Waits for VBlank before presenting, but multiple frames may be submitted before the VBlank occurs, does not cap the framerate, no tearing should occur",
        PresentMode::Immediate => "Does not wait for VBlank before presenting, does not cap the framerate, but visible tearing may occur",
    }
}

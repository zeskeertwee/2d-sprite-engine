use egui::{CtxRef, Rgba, Vec2};
use egui_wgpu_backend::epi::{Frame, Storage};
use std::time::Duration;
use wgpu::PresentMode;

pub mod integration;

pub struct DebugUi {
    /// the frametime, in seconds
    frametime: f64,
    present_mode: PresentMode,
}

impl epi::App for DebugUi {
    fn name(&self) -> &str {
        "Debug UI for sprite-engine"
    }

    fn update(&mut self, ctx: &CtxRef, _frame: &mut Frame<'_>) {
        egui::Window::new("Debug").show(ctx, |ui| {
            ui.label(format!(
                "Frametime: {:.2}ms {}",
                self.frametime * 1000.0,
                if self.present_mode == PresentMode::Mailbox {
                    "POSSIBLY INACCURATE"
                } else {
                    ""
                }
            ));
            ui.add_space(15.0);
            ui.label(format!(
                "FPS: {:.2} {}",
                1.0 / self.frametime,
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
                    ui.selectable_value(
                        &mut self.present_mode,
                        PresentMode::Immediate,
                        "Immediate",
                    );
                });
            ui.label(present_mode_description(self.present_mode));
        });
    }
}

impl Default for DebugUi {
    fn default() -> Self {
        Self {
            frametime: 0.0,
            present_mode: PresentMode::Fifo,
        }
    }
}

impl DebugUi {
    pub fn set_frametime(&mut self, frametime: f64) {
        self.frametime = frametime;
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

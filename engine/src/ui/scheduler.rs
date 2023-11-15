use crate::scheduler::{JobScheduler, ThreadState};
use crate::ui::{EguiWindow, MenuCategory};
use egui::{Grid, ScrollArea, Ui};

#[derive(Default)]
pub struct SchedulerWorkerThreadWindow;

impl EguiWindow for SchedulerWorkerThreadWindow {
    fn title(&self) -> &'static str {
        "Job threads"
    }

    fn menu_category(&self) -> MenuCategory {
        MenuCategory::Debug
    }

    fn draw(&mut self, ui: &mut Ui) {
        puffin::profile_function!("SchedulerWorkerThreadWindow");
        let states = JobScheduler::thread_states();

        ScrollArea::new([false, true]).show(ui, |ui| {
            ui.label(format!(
                "{} threads are idle",
                states.iter().filter(|s| *s == &ThreadState::Idle).count(),
            ));
            ui.add_space(15.0);

            Grid::new("thread_state_grid")
                .num_columns(2)
                .spacing([60.0, 4.0])
                .show(ui, |ui| {
                    for (i, state) in states.iter().enumerate() {
                        ui.label(format!("Thread {}", i));
                        ui.label(format!("{:?}", state));
                        ui.end_row();
                    }
                });
        })
    }
}

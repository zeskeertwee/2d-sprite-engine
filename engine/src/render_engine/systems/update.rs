use crate::render_engine::RenderEngineResources;
use bevy_ecs::prelude::*;
use log::info;
pub fn update_render_engine(mut engine: ResMut<RenderEngineResources>) {
    puffin::profile_function!();
    engine.camera.update_uniform_buffer(&engine.queue);

    if engine.config.present_mode != engine.egui_debug_ui.read().fps_window().present_mode() {
        info!(
            "Updating present mode from {:?} to {:?} (reason: DebugUi)",
            engine.config.present_mode,
            engine.egui_debug_ui.read().fps_window().present_mode()
        );
        let mode = engine
            .egui_debug_ui
            .read()
            .fps_window()
            .present_mode()
            .clone();
        engine.config.present_mode = mode;
        engine.reconfigure_surface();
    }

    engine.egui_debug_ui.write().cache_window_mut().update(
        &engine.device,
        engine.egui_integration.lock().render_pass_mut(),
    );
}

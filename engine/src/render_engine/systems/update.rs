use crate::render_engine::RenderEngineResources;
use bevy_ecs::prelude::*;
use std::sync::Arc;

pub fn update_render_engine(mut engine: ResMut<RenderEngineResources>) {
    engine.camera.update_uniform_buffer(&engine.queue);
    let frametime = engine.total_frame_time().as_secs_f64();
    engine
        .egui_debug_ui
        .get_mut()
        .fps_window_mut()
        .set_frametime(frametime);

    if engine.config.present_mode != engine.egui_debug_ui.read().fps_window().present_mode() {
        log::info!(
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

    let dev = Arc::clone(&engine.device);
    engine
        .egui_debug_ui
        .write()
        .cache_window_mut()
        .update(&dev, engine.egui_integration.lock().render_pass_mut());
}

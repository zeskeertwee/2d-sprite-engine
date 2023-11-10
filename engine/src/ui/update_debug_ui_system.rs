use crate::ecs::resources::DeltaTime;
use crate::render_engine::RenderEngineResources;
use bevy_ecs::system::Res;

pub fn update_debug_ui_frametime(engine: Res<RenderEngineResources>, dt: Res<DeltaTime>) {
    puffin::profile_function!();
    engine.with_debug_ui(|d| {
        d.fps_window_mut().set_frametime(dt.0.as_secs_f64());
    })
}

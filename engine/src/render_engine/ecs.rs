use crate::ecs::ScheduleStages;
use crate::render_engine::RenderEngineResources;
use crate::ui::integration::EguiRequestRedrawEvent;
use bevy_ecs::prelude::*;
use winit::event_loop::EventLoop;
use winit::window::Window;

use super::systems;

pub fn init_renderer_resources_in_world(
    world: &mut World,
    window: Window,
    event_loop: &EventLoop<EguiRequestRedrawEvent>,
) {
    let engine_res = RenderEngineResources::new(window, event_loop);
    world.insert_resource(engine_res);
}

pub fn insert_renderer_systems_in_schedule(schedule: &mut Schedule) {
    schedule.add_system_to_stage(
        ScheduleStages::Update.to_str(),
        systems::update::update_render_engine,
    );

    schedule.add_system_to_stage(ScheduleStages::Render.to_str(), systems::render::ecs_render);
}

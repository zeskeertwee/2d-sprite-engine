use crate::ecs::EcsWorld;
use crate::render_engine::resources::CommandBufferCollector;
use crate::render_engine::RenderEngineResources;
use crate::ui::integration::EguiRequestRedrawEvent;
use bevy_ecs::prelude::*;
use winit::event_loop::EventLoop;
use winit::window::Window;

use super::systems;

pub fn init_renderer_resources_in_world(
    world: &mut EcsWorld,
    window: Window,
    event_loop: &EventLoop<EguiRequestRedrawEvent>,
) {
    let engine_res = RenderEngineResources::new(window, event_loop);
    world.world.insert_resource(engine_res);
    world
        .world
        .insert_resource(CommandBufferCollector::default());
}

pub fn insert_renderer_systems_in_schedule(world: &mut EcsWorld) {
    //world.schedule.add_system_to_stage(
    //    ScheduleStages::Update.to_str(),
    //    systems::update::update_render_engine,
    //);
    //
    //world.schedule.add_system_to_stage(
    //    ScheduleStages::InternalPreRender.to_str(),
    //    systems::create_frame::ecs_render_create_frame_resource,
    //);
    //
    //world.schedule.add_system_to_stage(
    //    ScheduleStages::InternalRender.to_str(),
    //    systems::render_sprites::ecs_render_sprites,
    //);
    //
    //world.schedule.add_system_to_stage(
    //    ScheduleStages::InternalRender.to_str(),
    //    systems::render_egui_ui::ecs_render_egui_ui,
    //);
    //
    //world.schedule.add_system_to_stage(
    //    ScheduleStages::InternalPostRender.to_str(),
    //    systems::submit_commands::ecs_render_submit_commands,
    //);

    world.schedule.add_systems((
        systems::update::update_render_engine,
        systems::create_frame::ecs_render_create_frame_resource
            .after(systems::update::update_render_engine),
    ));

    world.render_schedule.add_systems((
        systems::render_sprites::ecs_render_sprites
            .after(systems::create_frame::ecs_render_create_frame_resource),
        systems::render_egui_ui::ecs_render_egui_ui
            .after(systems::create_frame::ecs_render_create_frame_resource)
            // TODO this is a really bad way to fix this, but it should now always draw the UI over the sprites
            .after(systems::render_sprites::ecs_render_sprites),
        systems::submit_commands::ecs_render_submit_commands
            .after(systems::render_sprites::ecs_render_sprites)
            .after(systems::render_egui_ui::ecs_render_egui_ui),
    ));
}

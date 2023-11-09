use crate::ecs::{EcsWorld, ScheduleStages};
use crate::render_engine::resources::FrameResources;
use crate::render_engine::RenderEngineResources;
use crate::ui::integration::EguiRequestRedrawEvent;
use parking_lot::Mutex;
use wgpu::CommandBuffer;
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
    let command_vec: Mutex<Vec<CommandBuffer>> = Mutex::new(Vec::new());
    world.world.insert_resource(command_vec);
    let frame_resource: Option<FrameResources> = None;
    world.world.insert_resource(frame_resource);
}

pub fn insert_renderer_systems_in_schedule(world: &mut EcsWorld) {
    world.schedule.add_system_to_stage(
        ScheduleStages::Update.to_str(),
        systems::update::update_render_engine,
    );

    world.schedule.add_system_to_stage(
        ScheduleStages::InternalPreRender.to_str(),
        systems::create_frame::ecs_render_create_frame_resource,
    );

    world.schedule.add_system_to_stage(
        ScheduleStages::InternalRender.to_str(),
        systems::render_sprites::ecs_render_sprites,
    );

    world.schedule.add_system_to_stage(
        ScheduleStages::InternalRender.to_str(),
        systems::render_egui_ui::ecs_render_egui_ui,
    );

    world.schedule.add_system_to_stage(
        ScheduleStages::InternalPostRender.to_str(),
        systems::submit_commands::ecs_render_submit_commands,
    );
}

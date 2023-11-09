pub mod resources;
mod stage;
mod systems;

use crate::ecs::resources::KeyboardInput;
use crate::render_engine::RenderEngineResources;
use bevy_ecs::prelude::*;
use bevy_ecs::world::EntityMut;
use cgmath::Vector2;
use log::warn;
pub use stage::ScheduleStages;
use winit::event::ElementState;

pub struct EcsWorld {
    pub world: World,
    pub schedule: Schedule,
}

impl EcsWorld {
    pub fn new() -> Self {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        // initialize stages
        for stage in stage::STAGES {
            schedule.add_stage(stage.to_str(), SystemStage::parallel());
        }
        //schedule.add_stage(ScheduleStages::PreUpdate.to_str(), SystemStage::parallel());
        //schedule.add_stage(ScheduleStages::Update.to_str(), SystemStage::parallel());
        //schedule.add_stage(ScheduleStages::PostUpdate.to_str(), SystemStage::parallel());
        //schedule.add_stage(ScheduleStages::PreRender.to_str(), SystemStage::parallel());
        //schedule.add_stage(
        //    ScheduleStages::InternalPreRender.to_str(),
        //    SystemStage::parallel(),
        //);
        //schedule.add_stage(
        //    ScheduleStages::InternalRender.to_str(),
        //    SystemStage::parallel(),
        //);
        //schedule.add_stage(
        //    ScheduleStages::InternalPostRender.to_str(),
        //    SystemStage::parallel(),
        //);
        //schedule.add_stage(ScheduleStages::PostRender.to_str(), SystemStage::parallel());

        // initialize resources
        world.insert_resource(systems::clean_cache::LastCacheClean::default());
        world.insert_resource(systems::clean_cache::CacheCleanInterval::default());
        world.insert_resource(resources::CursorPosition::default());
        world.insert_resource(resources::KeyboardInput::default());
        world.insert_resource(resources::DeltaTime::default());
        world.insert_resource(resources::delta_time::LastDeltaTimeInstant::default());

        // initialize systems
        schedule.add_system_to_stage(
            ScheduleStages::PreUpdate.to_str(),
            systems::delta_time::update_delta_time,
        );

        schedule.add_system_to_stage(
            ScheduleStages::Update.to_str(),
            crate::ui::update_debug_ui_system::update_debug_ui_frametime,
        );
        schedule.add_system_to_stage(
            ScheduleStages::Update.to_str(),
            systems::clean_cache::clean_cache,
        );

        Self { world, schedule }
    }

    pub fn run_schedule(&mut self) {
        puffin::profile_function!();
        for (idx, stage) in stage::STAGES.iter().enumerate() {
            puffin::profile_scope!(stage::PROFILER_STAGE_NAMES[idx]);
            let stage: &mut SystemStage = self.schedule.get_stage_mut(&stage.to_str()).unwrap();
            stage.run(&mut self.world);
        }
    }

    pub fn insert_entity<F: FnOnce(EntityMut) -> R, R>(&mut self, f: F) -> R {
        let entity = self.world.spawn();
        (f)(entity)
    }

    pub fn get_entity_mut<F: FnOnce(EntityMut) -> R, R>(&mut self, entity: Entity, f: F) -> R {
        (f)(self.world.get_entity_mut(entity).unwrap())
    }

    pub fn get_render_engine<F: FnOnce(Mut<RenderEngineResources>) -> R, R>(&mut self, f: F) -> R {
        (f)(self.world.get_resource_mut().unwrap())
    }

    pub fn update_cursor_position(&mut self, pos: Vector2<f32>) {
        let world_space = self.get_render_engine(|e| e.camera().mouse_pos_to_world_space(pos));
        self.world.insert_resource(resources::CursorPosition {
            screen_space: pos,
            world_space,
        });
    }

    pub fn update_keyboard_input(&mut self, input: winit::event::KeyboardInput) {
        let mut input_res: Mut<KeyboardInput> = self.world.get_resource_mut().unwrap();

        if let Some(keycode) = input.virtual_keycode {
            match input.state {
                ElementState::Pressed => input_res.add_pressed(keycode),
                ElementState::Released => input_res.remove_pressed(&keycode),
            }
        } else {
            warn!("Keyboard input event without keycode?");
        }
    }
}

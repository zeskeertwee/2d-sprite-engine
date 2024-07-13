pub mod resources;
mod stage;
pub mod systems;

use crate::ecs::resources::KeyboardInput;
use crate::render_engine::RenderEngineResources;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ExecutorKind;
use bevy_ecs::world::EntityMut;
use cgmath::Vector2;
use log::warn;
pub use stage::ScheduleStages;
use winit::event::ElementState;

pub struct EcsWorld {
    pub world: World,
    pub schedule: Schedule,
    pub render_schedule: Schedule,
}

impl EcsWorld {
    pub fn new() -> Self {
        let mut world = World::default();
        let mut schedule = Schedule::default();
        schedule.set_executor_kind(ExecutorKind::MultiThreaded);
        let mut render_schedule = Schedule::default();
        render_schedule.set_executor_kind(ExecutorKind::MultiThreaded);

        // initialize resources
        world.insert_resource(systems::clean_cache::LastCacheClean::default());
        world.insert_resource(systems::clean_cache::CacheCleanInterval::default());
        world.insert_resource(resources::CursorPosition::default());
        world.insert_resource(resources::KeyboardInput::default());
        world.insert_resource(resources::DeltaTime::default());
        world.insert_resource(resources::delta_time::LastDeltaTimeInstant::default());

        // initialize systems
        schedule.add_systems((
            systems::delta_time::update_delta_time,
            crate::ui::update_debug_ui_system::update_debug_ui_frametime
                .after(systems::delta_time::update_delta_time),
            // we don't care when this runs
            systems::clean_cache::clean_cache,
        ));

        let mut world = Self {
            world,
            schedule,
            render_schedule,
        };

        world
    }

    pub fn run_schedule(&mut self) {
        puffin::profile_function!();
        self.schedule.run(&mut self.world);
        self.schedule.apply_deferred(&mut self.world);
        self.render_schedule.run(&mut self.world);
    }

    pub fn insert_entity<F: FnOnce(EntityWorldMut) -> R, R>(&mut self, f: F) -> R {
        let entity = self.world.spawn_empty();
        (f)(entity)
    }

    pub fn get_entity_mut<F: FnOnce(EntityWorldMut) -> R, R>(&mut self, entity: Entity, f: F) -> R {
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

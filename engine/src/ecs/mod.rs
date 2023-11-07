mod stage;

use bevy_ecs::prelude::*;
pub use stage::ScheduleStages;

pub struct EcsWorld {
    pub world: World,
    pub schedule: Schedule,
}

impl EcsWorld {
    pub fn new() -> Self {
        let mut world = World::default();
        let mut schedule = Schedule::default();

        schedule.add_stage(ScheduleStages::PreUpdate.to_str(), SystemStage::parallel());
        schedule.add_stage(ScheduleStages::Update.to_str(), SystemStage::parallel());
        schedule.add_stage(ScheduleStages::PostUpdate.to_str(), SystemStage::parallel());
        schedule.add_stage(ScheduleStages::PreRender.to_str(), SystemStage::parallel());
        schedule.add_stage(ScheduleStages::Render.to_str(), SystemStage::parallel());
        schedule.add_stage(ScheduleStages::PostRender.to_str(), SystemStage::parallel());

        Self { world, schedule }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ScheduleStages {
    PreUpdate,
    Update,
    PostUpdate,
    PreRender,
    InternalPreRender,
    InternalRender,
    InternalPostRender,
    PostRender,
}

pub const STAGES: [ScheduleStages; 8] = [
    ScheduleStages::PreUpdate,
    ScheduleStages::Update,
    ScheduleStages::PostUpdate,
    ScheduleStages::PreRender,
    ScheduleStages::InternalPreRender,
    ScheduleStages::InternalRender,
    ScheduleStages::InternalPostRender,
    ScheduleStages::PostRender,
];

pub const PROFILER_STAGE_NAMES: [&'static str; 8] = [
    "ECS-Stage-PreUpdate",
    "ECS-Stage-Update",
    "ECS-Stage-PostUpdate",
    "ECS-Stage-PreRender",
    "ECS-Stage-InternalPreRender",
    "ECS-Stage-InternalRender",
    "ECS-Stage-InternalPostRender",
    "ECS-Stage-PostRender",
];

impl ScheduleStages {
    pub fn to_str(&self) -> &'static str {
        match self {
            ScheduleStages::PreUpdate => "pre_update",
            ScheduleStages::Update => "update",
            ScheduleStages::PostUpdate => "post_update",
            ScheduleStages::PreRender => "pre_render",
            ScheduleStages::InternalPreRender => "i_prerender",
            ScheduleStages::InternalRender => "i_render",
            ScheduleStages::InternalPostRender => "i_postrender",
            ScheduleStages::PostRender => "post_render",
        }
    }
}

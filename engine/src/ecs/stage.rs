#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ScheduleStages {
    PreUpdate,
    Update,
    PostUpdate,
    PreRender,
    Render,
    PostRender,
}

impl ScheduleStages {
    pub fn to_str(&self) -> &'static str {
        match self {
            ScheduleStages::PreUpdate => "pre_update",
            ScheduleStages::Update => "update",
            ScheduleStages::PostUpdate => "post_update",
            ScheduleStages::PreRender => "pre_render",
            ScheduleStages::Render => "render",
            ScheduleStages::PostRender => "post_render",
        }
    }
}

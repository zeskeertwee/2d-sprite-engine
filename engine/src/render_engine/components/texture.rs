use crate::asset_management::GpuTextureRef;
use bevy_ecs::component::Component;

#[derive(Component)]
pub struct Texture(pub GpuTextureRef);

use bevy_ecs::component::Component;
use cgmath::Vector3;

#[derive(Component)]
pub struct Position(pub Vector3<f32>);
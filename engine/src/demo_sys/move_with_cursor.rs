use crate::ecs::resources::CursorPosition;
use crate::render_engine::components::position::Position;
use bevy_ecs::component::Component;
use bevy_ecs::system::{Query, Res};
use cgmath::Vector2;

#[derive(Component)]
pub struct MoveWithCursor {
    pub(crate) offset: Vector2<f32>,
}

pub fn move_with_cursor_sys(
    mouse_pos: Res<CursorPosition>,
    mut entities: Query<(&mut Position, &MoveWithCursor)>,
) {
    for (mut pos, movewith) in entities.iter_mut() {
        let new_pos = mouse_pos.world_space + movewith.offset;
        pos.0.x = new_pos.x;
        pos.0.y = new_pos.y;
    }
}

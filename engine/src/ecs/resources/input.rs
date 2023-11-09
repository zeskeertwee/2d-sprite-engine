use cgmath::Vector2;
use std::collections::HashSet;
use winit::event::VirtualKeyCode;

pub struct CursorPosition {
    pub screen_space: Vector2<f32>,
    pub world_space: Vector2<f32>,
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self {
            screen_space: Vector2::new(0.0, 0.0),
            world_space: Vector2::new(0.0, 0.0),
        }
    }
}

pub struct KeyboardInput {
    pressed: HashSet<VirtualKeyCode>,
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self {
            pressed: HashSet::new(),
        }
    }
}

impl KeyboardInput {
    pub fn is_pressed(&self, keycode: &VirtualKeyCode) -> bool {
        self.pressed.contains(keycode)
    }

    pub fn add_pressed(&mut self, keycode: VirtualKeyCode) {
        self.pressed.insert(keycode);
    }

    pub fn remove_pressed(&mut self, keycode: &VirtualKeyCode) {
        self.pressed.remove(keycode);
    }
}

use std::collections::HashSet;
use wasm_bindgen::prelude::wasm_bindgen;
use math::Vector2D;

#[derive(Debug, Default)]
pub struct InputState {
    pub mouse: Vector2D,
    pub keyboard: HashSet<Key>,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    Spacebar,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    W,
    A,
    S,
    D,
    LShift,
}

impl InputState {
    pub fn on_mouse_move(&mut self, mouse_x: f32, mouse_y: f32) {
        self.mouse = Vector2D::new(mouse_x, mouse_y);
    }

    pub fn on_key_down(&mut self, key: Key) {
        self.keyboard.insert(key);
    }

    pub fn on_key_up(&mut self, key: Key) {
        self.keyboard.remove(&key);
    }

    pub fn is_down(&self, key: Key) -> bool {
        self.keyboard.contains(&key)
    }
}

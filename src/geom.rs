use specs::{Component, VecStorage, World};

#[derive(Copy, Clone)]
pub struct Position {
    x: f32,
    y: f32
}

impl Position {
    pub fn as_i32(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}

impl Component for Position {
    type Storage = VecStorage<Self>;
}

pub fn register(world: &mut World) {
    world.register::<Position>();
}

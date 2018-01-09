use specs::{Component, VecStorage};
use geom::Position;
use blit::*;

pub struct Sprite(BlitBuffer);

impl Sprite {
    pub fn new(buf: BlitBuffer) -> Self {
        Sprite(buf)
    }

    pub fn draw(&self, pos: &Position, buffer: &mut Vec<u32>, buffer_size: (usize, usize)) {
        self.0.blit(buffer, buffer_size, pos.as_i32());
    }
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}

extern crate image;

use specs::{Component, VecStorage};
use geom::Position;
use blit::*;

const MASK_COLOR: u32 = 0xFFFF00FF;

pub struct Sprite(BlitBuffer);

impl Sprite {
    pub fn new(file: &'static str) -> Self {
        let dynamic_img = image::open(file).unwrap();
        let img = dynamic_img.as_rgb8().unwrap();

        Sprite(img.as_blit_buffer(MASK_COLOR))
    }

    pub fn draw(&self, pos: &Position, buffer: &mut Vec<u32>, buffer_size: (usize, usize)) {
        self.0.blit(buffer, buffer_size, pos.as_i32());
    }
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}

extern crate image;

use image::RgbImage;
use specs::{Component, VecStorage};
use geom::Position;
use blit::*;

const MASK_COLOR: u32 = 0xFFFF00FF;

pub struct Sprite {
    img: RgbImage,

    mask: u32
}

impl Sprite {
    pub fn new(file: &'static str) -> Self {
        let dynamic_img = image::open(file).unwrap();
        let img = *dynamic_img.as_rgb8().unwrap();

        Sprite {
            img,
            mask: MASK_COLOR
        }
    }

    pub fn blit(&mut self, pos: &Position, buffer: &mut Vec<u32>, buffer_size: (usize, usize)) {
        self.img.blit_with_mask_color(buffer, buffer_size, pos.as_i32(), self.mask);
    }
}

impl Component for Sprite {
    type Storage = VecStorage<Self>;
}

use specs::*;
use physics::Position;
use blit::*;
use std::error::Error;

#[derive(Component)]
pub struct Sprite {
    pub pos: Position,
    img_ref: usize
}

impl Sprite {
    pub fn new(img_ref: usize) -> Self {
        Sprite {
            img_ref,
            pos: Position::new(0.0, 0.0)
        }
    }

    pub fn img_ref(&self) -> usize {
        self.img_ref
    }
}

pub struct SpriteSystem;

impl<'a> System<'a> for SpriteSystem {
    type SystemData = (ReadStorage<'a, Position>,
                       WriteStorage<'a, Sprite>);

    fn run(&mut self, (pos, mut sprite): Self::SystemData) {
        for (pos, sprite) in (&pos, &mut sprite).join() {
            sprite.pos = *pos;
        }
    }
}

#[derive(Component)]
pub struct Mask {
    pub pos: Position,
    mask_ref: usize
}

impl Mask {
    pub fn new(mask_ref: usize) -> Self {
        Mask {
            mask_ref,
            pos: Position::new(0.0, 0.0)
        }
    }

    pub fn mask_ref(&self) -> usize {
        self.mask_ref
    }
}

pub struct MaskSystem;

impl<'a> System<'a> for MaskSystem {
    type SystemData = (ReadStorage<'a, Position>,
                       WriteStorage<'a, Mask>);

    fn run(&mut self, (pos, mut mask): Self::SystemData) {
        for (pos, sprite) in (&pos, &mut mask).join() {
            sprite.pos = *pos;
        }
    }
}

pub struct Render {
    background: Vec<u32>,
    foreground: Vec<u32>,
    terrain: Vec<u32>,

    sprite_buffers: Vec<BlitBuffer>,

    width: usize,
    height: usize,
}

impl Render {
    pub fn new(size: (usize, usize)) -> Self {
        Render {
            background: vec![0; size.0 * size.1],
            foreground: vec![0xFFFF00FF; size.0 * size.1],
            terrain: vec![0xFFFF00FF; size.0 * size.1],

            width: size.0,
            height: size.1,

            sprite_buffers: Vec::new()
        }
    }

    pub fn draw_final_buffer(&mut self, buffer: &mut Vec<u32>) {
        for (output, (bg, (fg, terrain))) in buffer.iter_mut().zip(self.background.iter().zip(self.foreground.iter_mut().zip(&self.terrain))) {
            if (*fg & 0xFFFFFF) != 0xFF00FF {
                *output = *fg;
                *fg = 0xFF00FF;
                continue;
            }
            if (*terrain & 0xFFFFFF) != 0xFF00FF {
                *output = *terrain;
                continue;
            }
            *output = *bg;
        }
    }

    pub fn draw_foreground(&mut self, sprite: &Sprite) -> Result<(), Box<Error>> {
        let buf = &self.sprite_buffers[sprite.img_ref()];

        let size = self.size();
        buf.blit(&mut self.foreground, size, sprite.pos.as_i32());

        Ok(())
    }

    pub fn draw_mask_terrain(&mut self, mask: &Mask) -> Result<(), Box<Error>> {
        let buf = &self.sprite_buffers[mask.mask_ref()];

        let size = self.size();
        buf.blit(&mut self.terrain, size, mask.pos.as_i32());

        Ok(())
    }

    pub fn draw_terrain_from_memory(&mut self, bytes: &[u8]) {
        let buf = BlitBuffer::load_from_memory(bytes).unwrap();

        buf.blit(&mut self.terrain, (self.width, self.height), (0, 0));
    }

    pub fn draw_background_from_memory(&mut self, bytes: &[u8]) {
        let buf = BlitBuffer::load_from_memory(bytes).unwrap();

        buf.blit(&mut self.background, (self.width, self.height), (0, 0));
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn sprite_size(&self, sprite_ref: usize) -> Result<(usize, usize), Box<Error>> {
        let buf = &self.sprite_buffers[sprite_ref];

        Ok(buf.size())
    }

    pub fn add(&mut self, sprite: BlitBuffer) -> usize {
        self.sprite_buffers.push(sprite);

        self.sprite_buffers.len() - 1
    }

    pub fn add_from_memory(&mut self, bytes: &[u8]) -> usize {
        let sprite = BlitBuffer::load_from_memory(bytes).unwrap();

        self.add(sprite)
    }
}

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

pub struct Render {
    pub buffer: Vec<u32>,

    sprite_buffers: Vec<BlitBuffer>,

    width: usize,
    height: usize,
}

impl Render {
    pub fn new(size: (usize, usize)) -> Self {
        Render {
            buffer: vec![0; size.0 * size.1],

            width: size.0,
            height: size.1,

            sprite_buffers: Vec::new()
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn add(&mut self, sprite: BlitBuffer) -> usize {
        self.sprite_buffers.push(sprite);

        self.sprite_buffers.len() - 1
    }

    pub fn add_from_memory(&mut self, bytes: &[u8]) -> usize {
        let sprite = BlitBuffer::load_from_memory(bytes).unwrap();

        self.add(sprite)
    }

    pub fn draw(&mut self, sprite: &Sprite) -> Result<(), Box<Error>> {
        let buf = &self.sprite_buffers[sprite.img_ref()];

        let size = self.size();
        buf.blit(&mut self.buffer, size, sprite.pos.as_i32());

        Ok(())
    }
}

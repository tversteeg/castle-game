use specs::*;
use sprite::Sprite;
use geom::Position;

pub struct RenderSystem {
    buffer: Buffer
}

impl RenderSystem {
    pub fn new(size: (usize, usize)) -> Self {
        RenderSystem {
            buffer: Buffer::new(size)
        }
    }

    pub fn raw_buffer(&self) -> &Vec<u32> {
        &self.buffer.data
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (ReadStorage<'a, Position>,
                       ReadStorage<'a, Sprite>);

    fn run(&mut self, data: Self::SystemData) {
        let (pos, sprite) = data;

        let buffer_size = self.buffer.size();

        for (pos, sprite) in (&pos, &sprite).join() {
            sprite.draw(pos, &mut self.buffer.data, buffer_size);
        }
    }
}

pub struct Buffer {
    pub width: usize,
    pub height: usize,

    pub data: Vec<u32>
}

impl Buffer {
    pub fn new(size: (usize, usize)) -> Self {
        Buffer {
            width: size.0,
            height: size.1,

            data: vec![0; size.0 * size.1]
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

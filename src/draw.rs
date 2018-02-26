use specs::*;
use blit::*;
use std::error::Error;
use std::collections::HashMap;
use cgmath::Point2;
use line_drawing::Bresenham;

use terrain::*;
use geom::*;

#[derive(Component, Debug, Copy, Clone)]
pub struct PixelParticle {
    pub color: u32,
    pub life: f64,

    pub pos: Point2<usize>
}

impl PixelParticle {
    pub fn new(color: u32, life: f64) -> Self {
        PixelParticle {
            color, life,
            pos: Point2::new(0, 0)
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct MaskId {
    pub id: usize,
    pub size: (usize, usize)
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Sprite {
    pub pos: Point,
    img_ref: usize
}

impl Sprite {
    pub fn new(img_ref: usize) -> Self {
        Sprite {
            img_ref,
            pos: Point::new(0.0, 0.0)
        }
    }

    pub fn img_ref(&self) -> usize {
        self.img_ref
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Line {
    pub p1: Point2<usize>,
    pub p2: Point2<usize>,
    pub color: u32
}

impl Line {
    pub fn new(color: u32) -> Self {
        Line {
            color,
            p1: Point2 {x: 0, y: 0},
            p2: Point2 {x: 0, y: 0},
        }
    }
}

pub struct Images(pub HashMap<String, usize>);

pub struct SpriteSystem;
impl<'a> System<'a> for SpriteSystem {
    type SystemData = (ReadStorage<'a, WorldPosition>,
                       WriteStorage<'a, Sprite>);

    fn run(&mut self, (pos, mut sprite): Self::SystemData) {
        for (pos, sprite) in (&pos, &mut sprite).join() {
            sprite.pos = pos.0;
        }
    }
}

pub struct Render {
    background: Vec<u32>,
    foreground: Vec<u32>,

    blit_buffers: Vec<(String, BlitBuffer)>,

    width: usize,
    height: usize,
}

impl Render {
    pub fn new(size: (usize, usize)) -> Self {
        Render {
            background: vec![0; (size.0 * size.1) as usize],
            foreground: vec![0xFFFF00FF; (size.0 * size.1) as usize],

            width: size.0,
            height: size.1,

            blit_buffers: Vec::new()
        }
    }

    pub fn draw_final_buffer(&mut self, buffer: &mut Vec<u32>, terrain: &Terrain) {
        for (output, (bg, (fg, terrain))) in buffer.iter_mut().zip(self.background.iter().zip(self.foreground.iter_mut().zip(&terrain.buffer))) {
            if (*fg & 0xFFFFFF) != 0xFF00FF {
                // Draw the foreground and clear it immediately
                *output = *fg;
                *fg = 0xFF00FF;
                continue;
            }
            if (*terrain & 0xFFFFFF) != 0xFF00FF {
                // The terrain doesn't needs to be cleared
                *output = *terrain;
                continue;
            }
            *output = *bg;
        }
    }

    pub fn draw_foreground(&mut self, sprite: &Sprite) -> Result<(), Box<Error>> {
        let buf = &self.blit_buffers[sprite.img_ref()].1;

        let size = self.size();
        buf.blit(&mut self.foreground, size.0, sprite.pos.as_i32());

        Ok(())
    }

    pub fn draw_foreground_pixel(&mut self, pos: Point2<usize>, color: u32) {
        if pos.x >= self.width || pos.y >= self.height {
            return;
        }

        self.foreground[pos.x + pos.y * self.width] = color;
    }

    pub fn draw_foreground_line(&mut self, p1: Point2<usize>, p2: Point2<usize>, color: u32) {
        if (p1.x >= self.width || p2.y >= self.height) && (p2.x >= self.width || p2.y >= self.height) {
            return;
        }

        for (x, y) in Bresenham::new((p1.x as i32, p1.y as i32), (p2.x as i32, p2.y as i32)) {
            if x >= self.width as i32 || y >= self.height as i32 {
                continue;
            }

            self.foreground[x as usize + y as usize * self.width] = color;
        }
    }

    pub fn draw_mask_terrain(&mut self, terrain: &mut Terrain, mask: &TerrainMask) -> Result<(), Box<Error>> {
        let buf = &self.blit_buffers[mask.id].1;

        // Center the mask
        let mut pos = mask.pos;
        pos.0 -= buf.size().0 / 2;
        pos.1 -= buf.size().1 / 2;

        let size = self.size();
        buf.blit(&mut terrain.buffer, size.0, pos);

        Ok(())
    }

    pub fn draw_terrain_from_memory(&mut self, terrain: &mut Terrain, bytes: &[u8]) {
        let buf = BlitBuffer::from_memory(bytes).unwrap();

        let size = self.size();
        buf.blit(&mut terrain.buffer, size.0, (0, 0));
    }

    pub fn draw_background_from_memory(&mut self, bytes: &[u8]) {
        let buf = BlitBuffer::from_memory(bytes).unwrap();

        let size = self.size();
        buf.blit(&mut self.background, size.0, (0, 0));
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn add_buf(&mut self, name: &str, buf: BlitBuffer) -> usize {
        self.blit_buffers.push((String::from(name), buf));

        self.blit_buffers.len() - 1
    }

    pub fn add_buf_from_memory(&mut self, name: &str, bytes: &[u8]) -> usize {
        let buf = BlitBuffer::from_memory(bytes).unwrap();

        self.add_buf(name, buf)
    }
}

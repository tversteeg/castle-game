use blit::*;
use cgmath::Point2;
use line_drawing::Bresenham;
use specs::*;
use specs_derive::Component;
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

use crate::geom::*;
use crate::terrain::*;

const GREEN_BAR_COLOR: u32 = 0xFF_6A_BE_30;
const RED_BAR_COLOR: u32 = 0xFF_AC_32_33;

#[derive(Component, Debug, Copy, Clone)]
pub struct PixelParticle {
    pub color: u32,
    pub life: f64,

    pub pos: Point2<usize>,
}

impl PixelParticle {
    pub fn new(color: u32, life: f64) -> Self {
        PixelParticle {
            color,
            life,
            pos: Point2::new(0, 0),
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct MaskId {
    pub id: usize,
    pub size: (usize, usize),
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Sprite {
    pub pos: Point,
    img_ref: usize,
}

impl Sprite {
    pub fn new(img_ref: usize) -> Self {
        Sprite {
            img_ref,
            pos: Point::new(0.0, 0.0),
        }
    }

    pub fn img_ref(&self) -> usize {
        self.img_ref
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Anim {
    pub pos: Point,
    pub info: Animation,
    img_ref: usize,
}

impl Anim {
    pub fn new(img_ref: usize, info: Animation) -> Self {
        Anim {
            img_ref,
            info,
            pos: Point::new(0.0, 0.0),
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
    pub color: u32,
}

impl Line {
    pub fn new(color: u32) -> Self {
        Line {
            color,
            p1: Point2 { x: 0, y: 0 },
            p2: Point2 { x: 0, y: 0 },
        }
    }
}

pub struct Images(pub HashMap<String, usize>);

pub struct SpriteSystem;
impl<'a> System<'a> for SpriteSystem {
    type SystemData = (ReadStorage<'a, WorldPosition>, WriteStorage<'a, Sprite>);

    fn run(&mut self, (pos, mut sprite): Self::SystemData) {
        for (pos, sprite) in (&pos, &mut sprite).join() {
            sprite.pos = pos.0;
        }
    }
}

pub struct AnimSystem;
impl<'a> System<'a> for AnimSystem {
    type SystemData = (ReadStorage<'a, WorldPosition>, WriteStorage<'a, Anim>);

    fn run(&mut self, (pos, mut anim): Self::SystemData) {
        for (pos, anim) in (&pos, &mut anim).join() {
            anim.pos = pos.0;
        }
    }
}

pub struct Render {
    background: Vec<u32>,

    blit_buffers: Vec<(String, BlitBuffer)>,
    anim_buffers: Vec<(String, AnimationBlitBuffer)>,

    width: usize,
    height: usize,
}

impl Render {
    pub fn new(size: (usize, usize)) -> Self {
        Render {
            background: vec![0; (size.0 * size.1) as usize],

            width: size.0,
            height: size.1,

            blit_buffers: Vec::new(),
            anim_buffers: Vec::new(),
        }
    }

    pub fn draw_terrain_and_background(&mut self, buffer: &mut Vec<u32>, terrain: &Terrain) {
        for (output, (bg, terrain)) in buffer
            .iter_mut()
            .zip(self.background.iter().zip(&terrain.buffer))
        {
            if (*terrain & 0xFF_FF_FF) != 0xFF_00_FF {
                // The terrain doesn't needs to be cleared
                *output = *terrain;
                continue;
            }
            *output = *bg;
        }
    }

    pub fn draw_healthbar(
        &mut self,
        buffer: &mut Vec<u32>,
        pos: Point2<usize>,
        health_ratio: f64,
        width: usize,
    ) {
        if pos.x >= self.width || pos.y >= self.height {
            return;
        }

        let y = pos.y * self.width;

        let width = if pos.x + width >= self.width {
            self.width - pos.x
        } else {
            width
        };
        let health = pos.x + (health_ratio * width as f64) as usize;

        // Draw the green bar
        for x in pos.x..health {
            buffer[x + y] = GREEN_BAR_COLOR;
        }

        // Draw the red bar
        let max = pos.x + width;
        for x in health..max {
            buffer[x + y] = RED_BAR_COLOR;
        }
    }

    pub fn draw_foreground(
        &mut self,
        buffer: &mut Vec<u32>,
        sprite: &Sprite,
    ) -> Result<(), Box<dyn Error>> {
        let buf = &self.blit_buffers[sprite.img_ref()].1;

        let size = self.size();
        buf.blit(buffer, size.0, sprite.pos.as_i32());

        Ok(())
    }

    pub fn draw_foreground_anim(
        &mut self,
        buffer: &mut Vec<u32>,
        anim: &Anim,
    ) -> Result<(), Box<dyn Error>> {
        let buf = &self.anim_buffers[anim.img_ref()].1;

        let size = self.size();
        buf.blit(buffer, size.0, anim.pos.as_i32(), &anim.info)?;

        Ok(())
    }

    pub fn draw_foreground_pixel(&mut self, buffer: &mut Vec<u32>, pos: Point2<usize>, color: u32) {
        if pos.x >= self.width || pos.y >= self.height {
            return;
        }

        buffer[pos.x + pos.y * self.width] = color;
    }

    pub fn draw_foreground_line(
        &mut self,
        buffer: &mut Vec<u32>,
        p1: Point2<usize>,
        p2: Point2<usize>,
        color: u32,
    ) {
        if p2.y >= self.height || p1.x >= self.width && p2.x >= self.width {
            return;
        }

        for (x, y) in Bresenham::new((p1.x as i32, p1.y as i32), (p2.x as i32, p2.y as i32)) {
            if x >= self.width as i32 || y >= self.height as i32 {
                continue;
            }

            buffer[x as usize + y as usize * self.width] = color;
        }
    }

    pub fn draw_mask_terrain(
        &mut self,
        terrain: &mut Terrain,
        mask: &TerrainMask,
    ) -> Result<(), Box<dyn Error>> {
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

    /// Update the animation with the buffer, this is needed here because the timings are described
    /// inside the AnimationBlitBuffer object.
    pub fn update_anim(&self, anim: &mut Anim, dt: Duration) -> Result<(), Box<dyn Error>> {
        let buf = &self.anim_buffers[anim.img_ref()].1;

        anim.info.update(buf, dt)?;

        Ok(())
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn add_buf_from_memory(&mut self, name: &str, bytes: &[u8]) -> usize {
        let buf = BlitBuffer::from_memory(bytes).unwrap();

        self.blit_buffers.push((String::from(name), buf));

        self.blit_buffers.len() - 1
    }

    pub fn add_anim_buf_from_memory(&mut self, name: &str, bytes: &[u8]) -> usize {
        let buf = AnimationBlitBuffer::from_memory(bytes).unwrap();

        self.anim_buffers.push((String::from(name), buf));

        self.anim_buffers.len() - 1
    }
}

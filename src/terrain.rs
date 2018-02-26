use specs::*;
use line_drawing::Bresenham;

use physics::*;
use geom::*;

pub struct Terrain {
    pub buffer: Vec<u32>,

    width: usize,
    height: usize
}

impl Terrain {
    pub fn new(size: (usize, usize)) -> Self {
        Terrain {
            buffer: vec![0xFFFF00FF; (size.0 * size.1) as usize],

            width: size.0,
            height: size.1,
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn line_collides(&self, start: (i32, i32), end: (i32, i32)) -> Option<(i32, i32)> {
        let (width, height) = self.size();

        for pos in Bresenham::new(start, end) {
            if pos.0 < 0 || pos.1 < 0 || pos.0 as usize >= width || pos.1 as usize >= height {
                continue;
            }

            let index = pos.0 as usize + pos.1 as usize * width;
            if (self.buffer[index] & 0xFFFFFF) != 0xFF00FF {
                return Some(pos);
            }
        }

        None
    }

    pub fn rect_collides(&self, rect: BoundingBox) -> Option<(i32, i32)> {
        let mut rect = rect.to_i32();

        // Clip the rectangle to the buffer
        if rect.0 < 0 {
            rect.2 += rect.0;
            rect.0 = 0;
        }
        if rect.1 < 0 {
            rect.3 += rect.1;
            rect.1 = 0;
        }

        let (width, height) = self.size();
        if rect.0 + rect.2 >= width as i32 {
            rect.2 = width as i32 - rect.0 - 1;
        }
        if rect.1 + rect.3 >= height as i32 {
            rect.3 = height as i32 - rect.1 - 1;
        }

        let start = (rect.0, rect.1);
        let end = (rect.0 + rect.2, rect.1 + rect.3);

        for y in start.1..end.1 {
            for x in start.0..end.0 {
                let index = x as usize + y as usize * width;
                if (self.buffer[index] & 0xFFFFFF) != 0xFF00FF {
                    return Some((x, y));
                }
            }
        }

        None
    }

    pub fn draw_pixel(&mut self, pos: (usize, usize), color: u32) {
        if pos.0 >= self.width || pos.1 >= self.height {
            return;
        }

        self.buffer[pos.0 + pos.1 * self.width] = color;
    }
}

#[derive(Component, Debug)]
pub struct TerrainMask {
    pub id: usize,
    pub pos: (i32, i32),
    pub size: (usize, usize)
}

impl TerrainMask {
    pub fn new(id: usize, pos: (i32, i32), size: (usize, usize)) -> Self {
        TerrainMask { id, pos, size }
    }
}

#[derive(Component, Debug)]
pub struct TerrainCollapse(pub BoundingBox);

pub struct TerrainCollapseSystem;
impl<'a> System<'a> for TerrainCollapseSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Terrain>,
                       WriteStorage<'a, TerrainCollapse>);

    fn run(&mut self, (entities, dt, _terrain, mut rect): Self::SystemData) {
        let dt = dt.to_seconds();

        for (entities, mut rect) in (&*entities, &mut rect).join() {
            
        }
    }
}

use line_drawing::Bresenham;

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
}

#[derive(Component, Debug)]
pub struct TerrainMask {
    pub id: usize,
    pub pos: (i32, i32)
}

impl TerrainMask {
    pub fn new(id: usize, pos: (i32, i32)) -> Self {
        TerrainMask { id, pos }
    }
}

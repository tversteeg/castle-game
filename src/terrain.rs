use vek::{Extent2, Vec2};

use crate::{camera::Camera, sprite::Sprite, SIZE};

/// Level asset path.
const ASSET_PATH: &str = "level.grass-1";

/// Destructible terrain buffer.
pub struct Terrain {
    /// Size of the terrain.
    size: Extent2<u32>,
    /// Array of the top collision point heights of the terrain.
    top_heights: Vec<u8>,
}

impl Terrain {
    /// Load a terrain from image bytes.
    pub fn new() -> Self {
        let sprite = crate::sprite(ASSET_PATH);
        let size = Extent2::new(sprite.width(), sprite.height());

        // Create an empty vector so we can fill it with a method
        let top_heights = vec![0; size.w as usize];

        let mut terrain = Self { size, top_heights };
        terrain.recalculate_top_height(&sprite);
        terrain
    }

    /// Draw the terrain based on a camera offset.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        puffin::profile_function!();

        crate::sprite(ASSET_PATH).render(canvas, camera, (0, self.y_offset()).into());
    }

    /// Whether an absolute coordinate hits the terrain.
    pub fn point_collides(&self, point: Vec2<i32>) -> bool {
        if point.y < self.y_offset() || point.x < 0 || point.x >= self.width() as i32 {
            false
        } else {
            // Collides if the top height is smaller than the position
            self.y_offset() + (self.top_heights[point.x as usize] as i32) < point.y
        }
    }

    /// Width of the terrain.
    pub fn width(&self) -> u32 {
        self.size.w
    }

    /// Total offset to place the terrain at the bottom.
    pub fn y_offset(&self) -> i32 {
        SIZE.h as i32 - self.size.h as i32
    }

    /// Recalculate the collision top heights.
    fn recalculate_top_height(&mut self, sprite: &Sprite) {
        puffin::profile_function!();

        // Loop over each X value
        self.top_heights
            .iter_mut()
            .enumerate()
            .for_each(|(x, height)| {
                // Loop over each Y value to find the first pixel that is not transparent
                *height = (0..sprite.height())
                    .find(|y| !sprite.is_pixel_transparent(x as u32, *y))
                    // If nothing is found just use the bottom
                    .unwrap_or(self.size.h) as u8;
            });
    }
}

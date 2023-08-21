use bitvec::vec::BitVec;
use vek::{Extent2, Vec2};

use crate::{
    graphics::Color,
    sprite::{Sprite, SpriteOffset},
};

/// Size of the outline in pixels.
const OUTLINE_SIZE: usize = 2;

/// Procedurally generatable shape with a solid color and an outline.
///
/// Will automatically recreate the sprite when the shape gets changed.
/// Splitting of the sprite can also be detected.
pub struct SolidShape {
    /// Shape that generates the sprite and the collider.
    shape: BitVec,
    /// Size of the shape.
    size: Extent2<usize>,
    /// Sprite offset for placing the middle point.
    offset: SpriteOffset,
    /// Color for the fill.
    fill_color: Color,
    /// Color for the outline.
    ///
    /// Outline is assumed to be 2 pixels big.
    outline_color: Color,
    /// Generated shape, cached in this struct so it can be referenced easily without regenerating.
    ///
    /// Will only be generated when accessed and dirty.
    sprite: Option<Sprite>,
}

impl SolidShape {
    /// Create the shape from a simple solid rectangle.
    pub fn from_rectangle(
        size_without_outline: Extent2<f64>,
        offset: SpriteOffset,
        fill_color: Color,
        outline_color: Color,
    ) -> Self {
        let size_without_outline = size_without_outline.as_();
        let size = size_without_outline + Extent2::new(OUTLINE_SIZE * 2, OUTLINE_SIZE * 2);

        let mut shape = BitVec::repeat(false, size.w * size.h);

        // Ignore the outline area
        for y in OUTLINE_SIZE..(OUTLINE_SIZE + size_without_outline.h) {
            let index_start = y * size.w + OUTLINE_SIZE;
            let index_end = index_start + size_without_outline.w;
            shape[index_start..index_end].fill(true);
        }

        Self {
            size,
            offset,
            shape,
            fill_color,
            outline_color,
            sprite: None,
        }
    }

    /// Create the shape as a heightmap where only the top edge has multiple subdivisions.
    ///
    /// Heights must be set per pixel.
    ///
    /// Height is the total height is added to the center Y.
    pub fn from_heights(
        heights: &[f64],
        height: f64,
        offset: SpriteOffset,
        fill_color: Color,
        outline_color: Color,
    ) -> Self {
        debug_assert!(!heights.is_empty());

        // Find the highest point so we know the max height
        let highest = heights
            .iter()
            .fold(std::f64::NEG_INFINITY, |a, &b| a.max(b));
        let total_width = heights.len();
        let total_height = (highest + height) as usize;
        let size = Extent2::new(total_width, total_height);

        // Create the vector
        let mut shape = BitVec::repeat(false, total_width * total_height);
        for (x, height) in heights.iter().enumerate() {
            // Fill every pixel from the height
            let start = highest - height + OUTLINE_SIZE as f64;
            for y in (start as usize)..total_height {
                shape.set(x + y * total_width, true);
            }
        }

        Self {
            size,
            offset,
            shape,
            fill_color,
            outline_color,
            sprite: None,
        }
    }

    /// Generate the sprite from the shape.
    pub fn generate_sprite(&mut self) {
        let mut sprite_buf = vec![0; self.size.w * self.size.h];
        for y in 0..self.size.h {
            let index_start = y * self.size.w;
            for x in 0..self.size.w {
                let index = index_start + x;
                if self.shape[index] {
                    // Solid color
                    sprite_buf[index] = self.fill_color.as_u32();
                } else if self.is_outline(x, y) {
                    // Solid color
                    sprite_buf[index] = self.outline_color.as_u32();
                }
            }
        }

        self.sprite = Some(Sprite::from_buffer(&sprite_buf, self.size, self.offset));
    }

    /// Get the sprite.
    ///
    /// Throws an error when [`Self::generate_sprite`] hasn't been called yet.
    pub fn sprite(&self) -> &Sprite {
        self.sprite
            .as_ref()
            .expect("Solid shape sprite not generated yet")
    }

    /// Whether a point collides.
    ///
    /// Assumes the point is in the local coordinate space of the shape.
    pub fn collides(&self, point: Vec2<f64>) -> bool {
        if point.x < 0.0 || point.y < 0.0 {
            // Out of bounds
            return false;
        }

        let x = point.x as usize;
        let y = point.y as usize;
        if x >= self.size.w || y >= self.size.h {
            // Out of bounds
            return false;
        }

        self.shape[x + y * self.size.w]
    }

    /// Whether a pixel in the shape should be an outline when rendering as a sprite.
    #[inline(always)]
    fn is_outline(&self, x: usize, y: usize) -> bool {
        // Shape of the outline, we don't check the middle coordinate since if that's solid it's not an outline
        let x = x as i32;
        let y = y as i32;
        let w = self.size.w as i32;
        let h = self.size.h as i32;
        for (offset_x, offset_y) in outline_offsets::OUTLINE_OFFSETS_2 {
            let x = x + offset_x;
            let y = y + offset_y;

            // Ensure we don't go out of bounds
            if x < 0 || x >= w || y < 0 || y >= h {
                continue;
            }

            // If we find any pixels that are solid we are an outline
            let index = (y * w + x) as usize;
            if self.shape[index] {
                return true;
            }
        }

        false
    }
}

#[rustfmt::skip]
mod outline_offsets {
    pub const OUTLINE_OFFSETS_2: [(i32, i32); 20] = [
                  (-1, -2), ( 0, -2), ( 1, -2),          
        (-2, -1), (-1, -1), ( 0, -1), ( 1, -1), ( 2, -1),
        (-2,  0), (-1,  0),           ( 1,  0), ( 2,  0),
        (-2,  1), (-1,  1), ( 0,  1), ( 1,  1), ( 2,  1),
                  (-1,  2), ( 0,  2), ( 1,  2),          
    ];
}

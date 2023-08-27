use bitvec::{slice::BitSlice, vec::BitVec};

use vek::{Extent2, Rect, Vec2};

use crate::{
    gen::isoline::Isoline,
    graphics::Color,
    physics::collision::shape::Shape,
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
    /// Color for the fill.
    fill_color: Color,
    /// Color for the outline.
    ///
    /// Outline is assumed to be 2 pixels big.
    outline_color: Color,
    /// Generated sprite from the shape with an outline.
    ///
    /// Must be updated whenever the shape is updated.
    sprite: Sprite,
    /// Generated collider from the shape.
    ///
    /// Must be updated whenever the shape is updated.
    collider: Isoline,
}

impl SolidShape {
    /// Create the shape from a simple solid rectangle.
    pub fn from_rectangle(
        size_without_outline: Extent2<f64>,
        offset: SpriteOffset,
        fill_color: Color,
        outline_color: Color,
    ) -> Self {
        puffin::profile_scope!("Solid shape from rectangle");

        let size_without_outline = size_without_outline.as_();
        let size = size_without_outline + Extent2::new(OUTLINE_SIZE * 2, OUTLINE_SIZE * 2);

        let mut shape = BitVec::repeat(false, size.w * size.h);

        // Ignore the outline area
        for y in OUTLINE_SIZE..(OUTLINE_SIZE + size_without_outline.h) {
            let index_start = y * size.w + OUTLINE_SIZE;
            let index_end = index_start + size_without_outline.w;
            shape[index_start..index_end].fill(true);
        }
        // Use an empty sprite, will be generated later
        let sprite = Sprite::from_buffer(&vec![0; size.w * size.h], size, offset);

        let collider = Isoline::from_bitmap(&shape, size);

        let mut this = Self {
            size,
            shape,
            fill_color,
            outline_color,
            sprite,
            collider,
        };

        this.generate_sprite();

        this
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
        puffin::profile_scope!("Solid shape from heights");

        debug_assert!(!heights.is_empty());

        // Find the highest point so we know the max height
        let highest = heights
            .iter()
            .fold(std::f64::NEG_INFINITY, |a, &b| a.max(b));
        let total_width = heights.len() + OUTLINE_SIZE * 2;
        let total_height = (highest + height) as usize + OUTLINE_SIZE * 2;
        let size = Extent2::new(total_width, total_height);

        // Create the vector
        let mut shape = BitVec::repeat(false, total_width * total_height);
        {
            puffin::profile_scope!("Fill shape");

            for (x, height) in heights.iter().enumerate() {
                // Fill every pixel from the height
                let start = highest - height + OUTLINE_SIZE as f64;
                for y in (start as usize)..(total_height - OUTLINE_SIZE) {
                    shape.set(x + OUTLINE_SIZE + y * total_width, true);
                }
            }
        }

        // Use an empty sprite, will be generated later
        let sprite = Sprite::from_buffer(&vec![0; size.w * size.h], size, offset);

        // Generate the first collider
        let collider = Isoline::from_bitmap(&shape, size);

        let mut this = Self {
            size,
            shape,
            fill_color,
            outline_color,
            sprite,
            collider,
        };

        this.generate_sprite();

        this
    }

    /// Generate the sprite from the shape.
    pub fn generate_sprite(&mut self) {
        puffin::profile_scope!("Generate sprite");

        // Redraw the full rectangle
        self.redraw_sprite_rectangle(self.rect());
    }

    /// Get the sprite.
    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    /// Get the collider shape.
    pub fn to_collider(&self) -> Shape {
        self.collider.to_collider(self.size.as_())
    }

    /// Whether a point collides.
    ///
    /// Assumes the point is in the local coordinate space of the shape.
    pub fn collides(&self, point: Vec2<f64>) -> bool {
        self.coord_to_index(point)
            .map(|index| self.shape[index])
            .unwrap_or_default()
    }

    /// Set a single pixel to transparent.
    pub fn remove_pixel(&mut self, pixel: Vec2<f64>) {
        puffin::profile_scope!("Remove pixel");

        let index = match self.coord_to_index(pixel) {
            Some(index) => index,
            None => return,
        };

        // Remove from the shape
        self.shape.set(index, false);

        // Update the sprite pixel affected
        self.set_sprite_pixel(pixel);

        // Update the sprite outline pixels as well
        outline_offsets::OUTLINE_OFFSETS_2
            .iter()
            .for_each(|(x, y)| self.set_sprite_pixel(pixel + Vec2::new(*x, *y).as_()));
    }

    /// Remove a circle of pixels.
    pub fn remove_circle(&mut self, center: Vec2<f64>, radius: f64) {
        puffin::profile_scope!("Remove circle");

        // The rectangle that will be used to redraw the collider and the sprite
        let rect = Rect::new(
            center.x - radius - 1.0,
            center.y - radius - 1.0,
            radius * 2.0 + 2.0,
            radius * 2.0 + 2.0,
        )
        .as_();

        // Delta bitmap with each pixel is removed inside the redraw area
        let mut removal_delta = BitVec::repeat(false, rect.w * rect.h);

        {
            puffin::profile_scope!("Create circle mask");
            // TODO: make this a lot more efficient
            let center = Vec2::new(radius, radius);
            for y in 0..rect.h {
                for x in 0..rect.w {
                    removal_delta.set(
                        y * rect.w + x,
                        Vec2::new(x as f64, y as f64).distance(center) < radius,
                    );
                }
            }
        }

        // Redraw a rectangle of the sprite
        self.apply_removal_mask(&removal_delta, rect);
    }

    /// Apply a bit vector of delta values which will remove pixels.
    fn apply_removal_mask(&mut self, removal_mask: &BitSlice, rect: Rect<usize, usize>) {
        puffin::profile_scope!("Apply removal deltas");

        debug_assert_eq!(removal_mask.len(), rect.w * rect.h);

        // Ensure it doesn't go out of bounds
        let rect = self.clamp_rect(rect);

        // Keep track of whether we updated any pixels
        let mut applied = false;
        // Apply to the shape
        for y in 0..rect.h {
            // Y start index on the removal delta map
            let delta_index = y * rect.w;
            // Y start index on the target shape map
            let shape_index = (y + rect.y) * self.size.w;

            for x in 0..rect.w {
                // PERF: use a bitwise operator and no loop here
                if removal_mask[delta_index + x] {
                    let index = shape_index + rect.x + x;
                    let pixel = self.shape.get_mut(index);
                    if let Some(mut pixel) = pixel {
                        // TODO: create map with updated pixels here
                        if !applied && *pixel {
                            applied = true;
                        }
                        *pixel = false;
                    }
                }
            }
        }

        if applied {
            // Redraw the sprite
            self.redraw_sprite_rectangle(rect);
            // Remove the part of the collider
            self.regenerate_collider_rectangle(removal_mask, rect);
        }
    }

    /// Redraw the sprite pixels of a rectangle, which will be clamped if outside of range.
    fn redraw_sprite_rectangle(&mut self, mut rect: Rect<usize, usize>) {
        puffin::profile_scope!("Redraw sprite rectangle");

        // Add the outline to the rectangle
        rect.x = rect.x.saturating_sub(OUTLINE_SIZE);
        rect.y = rect.y.saturating_sub(OUTLINE_SIZE);
        rect.w = (rect.w + OUTLINE_SIZE).min(self.size.w);
        rect.h = (rect.h + OUTLINE_SIZE).min(self.size.h);

        // Set the sprite pixels
        let min = rect.position();
        let max = rect.position() + (rect.w, rect.h);
        for y in min.y..max.y {
            let index_start = y * self.size.w;
            for x in min.x..max.x {
                let index = index_start + x;
                self.set_sprite_pixel_unchecked(index, Vec2::new(x, y));
            }
        }
    }

    /// Regenerate the rectangle part of the collider, which will be clamped if outside of range.
    fn regenerate_collider_rectangle(&mut self, removal_mask: &BitSlice, rect: Rect<usize, usize>) {
        puffin::profile_scope!("Regenerate collider");

        // PERF: cache isoline result
        self.collider
            .update(&self.shape, self.size, removal_mask, rect);
    }

    /// Convert a coordinate to a pixel index, returning nothing when out of bound.
    fn coord_to_index(&self, coord: Vec2<f64>) -> Option<usize> {
        if coord.x < 0.0 || coord.y < 0.0 {
            // Out of bounds
            return None;
        }

        let x = coord.x as usize;
        let y = coord.y as usize;
        if x >= self.size.w || y >= self.size.h {
            // Out of bounds
            None
        } else {
            Some(x + y * self.size.w)
        }
    }

    /// Set a sprite pixel if within bounds.
    #[inline(always)]
    fn set_sprite_pixel(&mut self, pixel: Vec2<f64>) {
        puffin::profile_scope!("Set sprite pixel");

        let index = match self.coord_to_index(pixel) {
            Some(index) => index,
            None => return,
        };

        // Set the pixel removed
        self.set_sprite_pixel_unchecked(index, pixel.as_());
    }

    /// Set a sprite pixel without checking the bounds.
    #[inline(always)]
    fn set_sprite_pixel_unchecked(&mut self, index: usize, pixel: Vec2<usize>) {
        puffin::profile_scope!("Set sprite pixel unchecked");

        self.sprite.pixels_mut()[index] = if self.shape[index] {
            // Solid fill
            self.fill_color.as_u32()
        } else if self.is_outline(pixel) {
            // Outline
            self.outline_color.as_u32()
        } else {
            0
        };
    }

    /// Whether a pixel in the shape should be an outline when rendering as a sprite.
    #[inline(always)]
    fn is_outline(&self, pos: Vec2<usize>) -> bool {
        // Shape of the outline, we don't check the middle coordinate since if that's solid it's not an outline
        let pos: Vec2<i32> = pos.as_();
        let size = self.size.as_();
        for (offset_x, offset_y) in outline_offsets::OUTLINE_OFFSETS_2 {
            let pos = pos + (offset_x, offset_y);

            // Ensure we don't go out of bounds
            if pos.x < 0 || pos.x >= size.w || pos.y < 0 || pos.y >= size.h {
                continue;
            }

            // If we find any pixels that are solid we are an outline
            let index = (pos.y * size.w + pos.x) as usize;
            if self.shape[index] {
                return true;
            }
        }

        false
    }

    /// Clamp a rectangle to the size of the buffer.
    pub fn clamp_rect(&self, mut rect: Rect<usize, usize>) -> Rect<usize, usize> {
        // Also clamp when the rectangle is at the edge
        rect.w = rect.w.min(self.size.w.saturating_sub(rect.x));
        rect.h = rect.h.min(self.size.h.saturating_sub(rect.y));

        rect
    }

    /// Get the rectangle for the full size.
    pub fn rect(&self) -> Rect<usize, usize> {
        Rect::new(0, 0, self.size.w, self.size.h)
    }
}

mod outline_offsets {
    #[rustfmt::skip]
    pub const OUTLINE_OFFSETS_2: [(i32, i32); 20] = [
                  (-1, -2), ( 0, -2), ( 1, -2),          
        (-2, -1), (-1, -1), ( 0, -1), ( 1, -1), ( 2, -1),
        (-2,  0), (-1,  0),           ( 1,  0), ( 2,  0),
        (-2,  1), (-1,  1), ( 0,  1), ( 1,  1), ( 2,  1),
                  (-1,  2), ( 0,  2), ( 1,  2),          
    ];
}

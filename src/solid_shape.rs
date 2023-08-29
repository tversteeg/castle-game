use vek::{Extent2, Rect, Vec2};

use crate::{
    gen::{bitmap::Bitmap, isoline::Isoline},
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
    shape: Bitmap,
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
        fill_color: Color,
        outline_color: Color,
    ) -> Self {
        puffin::profile_scope!("Solid shape from rectangle");

        let size_without_outline = size_without_outline.as_();
        let size = size_without_outline + Extent2::new(OUTLINE_SIZE * 2, OUTLINE_SIZE * 2);

        let mut shape = Bitmap::empty(size);

        // Ignore the outline area
        for y in OUTLINE_SIZE..(OUTLINE_SIZE + size_without_outline.h) {
            let index_start = y * size.w + OUTLINE_SIZE;
            let index_end = index_start + size_without_outline.w;
            shape.set_at_index_range(index_start..index_end, true);
        }

        // Use an empty sprite, will be generated later
        let sprite = Sprite::from_buffer(&vec![0; size.w * size.h], size, SpriteOffset::LeftTop);

        let collider = Isoline::from_bitmap(&shape);

        let mut this = Self {
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
        let mut shape = Bitmap::empty(size);
        {
            puffin::profile_scope!("Fill shape");

            for (x, height) in heights.iter().enumerate() {
                // Fill every pixel from the height
                let start = highest - height + OUTLINE_SIZE as f64;
                for y in (start as usize)..(size.h - OUTLINE_SIZE) {
                    shape.set(Vec2::new(x + OUTLINE_SIZE, y), true);
                }
            }
        }

        // Use an empty sprite, will be generated later
        let sprite = Sprite::from_buffer(&vec![0; size.w * size.h], size, SpriteOffset::LeftTop);

        // Generate the first collider
        let collider = Isoline::from_bitmap(&shape);

        let mut this = Self {
            shape,
            fill_color,
            outline_color,
            sprite,
            collider,
        };

        this.generate_sprite();

        this
    }

    /// Create from an existing bitmap.
    pub fn from_bitmap(mut shape: Bitmap, fill_color: Color, outline_color: Color) -> Self {
        puffin::profile_scope!("Solid shape from bitmap");

        debug_assert!(!shape.is_empty());

        // Make the memory layout efficient
        let offset = shape.shrink_with_padding(OUTLINE_SIZE);

        // Use an empty sprite, will be generated later
        let sprite = Sprite::from_buffer(
            &vec![0; shape.size().product()],
            shape.size(),
            SpriteOffset::Custom(offset.as_()),
        );

        // Generate the first collider
        let collider = Isoline::from_bitmap(&shape);

        let mut this = Self {
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
        self.collider.to_collider()
    }

    /// Whether a point collides.
    ///
    /// Assumes the point is in the local coordinate space of the shape.
    pub fn collides(&self, point: Vec2<f64>) -> bool {
        self.coord_to_index(point)
            .map(|index| self.shape[index])
            .unwrap_or_default()
    }

    /// Remove a circle of pixels.
    pub fn remove_circle(&mut self, center: Vec2<f64>, radius: f64) -> Vec<Self> {
        puffin::profile_scope!("Remove circle");

        // Do nothing if the circle is not within bounds
        if center.x < -radius
            || center.y < -radius
            || center.x > self.shape.width() as f64 + radius
            || center.y > self.shape.height() as f64 + radius
        {
            return Vec::new();
        }

        // Size is the same for both dimensions since it's a circle
        let size = radius as usize * 2 + OUTLINE_SIZE * 2;

        // Delta bitmap with each pixel is removed inside the redraw area
        let mut removal_mask = Bitmap::empty(Extent2::new(size, size));

        {
            puffin::profile_scope!("Create circle mask");

            // PERF: make this a lot more efficient
            let center = Vec2::new(size / 2, size / 2).as_();
            for y in 0..size {
                let y_index = y * size;
                for x in 0..size {
                    removal_mask.set_at_index(
                        y_index + x,
                        Vec2::new(x as f64, y as f64).distance(center) < radius,
                    );
                }
            }
        }

        // Redraw a rectangle of the sprite
        self.apply_removal_mask(
            &mut removal_mask,
            (center - (size as f64 / 2.0, size as f64 / 2.0)).as_(),
        )
    }

    /// Apply a bit vector of delta values which will remove pixels.
    fn apply_removal_mask(&mut self, removal_mask: &mut Bitmap, offset: Vec2<i32>) -> Vec<Self> {
        puffin::profile_scope!("Apply removal deltas");

        // Clip the removal mask to ignore edges
        let (mut offset, removal_mask) = removal_mask.clip(offset, self.shape.size());

        // Remove the removal mask from the shape, returning every pixel that got updated
        let mut delta_mask = self.shape.apply_removal_mask(&removal_mask, offset);
        if delta_mask.is_empty() {
            // No pixels to apply, so do nothing
            return Vec::new();
        }

        // Shrink the mask to fit the contents, making everything more efficient
        offset += delta_mask.shrink_with_padding(OUTLINE_SIZE);

        // Get the area of the delta mask on the shape itself
        let (_, shape_subsection) = self
            .shape
            .clip(Vec2::zero() - offset.as_(), delta_mask.size());

        // First do a small floodfill check on a small section to see if there are multiple islands
        // Then do a broad floodfill check on the whole shape
        let mut new_shapes = Vec::new();
        if shape_subsection.has_multiple_islands() && self.shape.has_multiple_islands() {
            puffin::profile_scope!("New shapes for islands");
            // Remove all islands with a floodfill
            while !self.shape.is_empty() {
                // Create a new shape from a floodfill
                let mut new_shape = self
                    .shape
                    .zeroing_floodfill_with_copy(self.shape.first_one().unwrap());

                // Make the shape more efficient
                new_shape.shrink_with_padding(OUTLINE_SIZE);

                new_shapes.push(Self::from_bitmap(
                    new_shape,
                    self.fill_color,
                    self.outline_color,
                ));
            }

            // Set current one to the largest shape
            let (largest_index, _) = new_shapes
                .iter()
                .enumerate()
                .max_by_key(|(_index, shape)| shape.shape.size().product())
                // Safe because there is a guarantee there exists at least two islands
                .unwrap();
            *self = new_shapes.remove(largest_index);

            new_shapes
        } else {
            // No new shapes found, do a partial update
            puffin::profile_scope!("Partial shape update");

            // Redraw the sprite
            self.redraw_sprite_rectangle(Rect::new(
                offset.x,
                offset.y,
                delta_mask.width(),
                delta_mask.height(),
            ));

            // Remove the part of the collider
            self.collider.update(&self.shape, &delta_mask, offset);

            Vec::new()
        }
    }

    /// Redraw the sprite pixels of a rectangle, which will be clamped if outside of range.
    fn redraw_sprite_rectangle(&mut self, rect: Rect<usize, usize>) {
        puffin::profile_scope!("Redraw sprite rectangle");

        debug_assert_eq!(self.shape.size(), self.sprite.size().as_());

        // Set the sprite pixels
        for y in 0..rect.h {
            let index_start = (y + rect.y) * self.shape.width();
            for x in 0..rect.w {
                let index = index_start + x + rect.x;
                self.set_sprite_pixel_unchecked(index, Vec2::new(x, y) + rect.position());
            }
        }
    }

    /// Convert a coordinate to a pixel index, returning nothing when out of bound.
    #[inline(always)]
    fn coord_to_index(&self, coord: Vec2<f64>) -> Option<usize> {
        if coord.x < 0.0 || coord.y < 0.0 {
            // Out of bounds
            return None;
        }

        let x = coord.x as usize;
        let y = coord.y as usize;
        if x >= self.shape.width() || y >= self.shape.height() {
            // Out of bounds
            None
        } else {
            Some(x + y * self.shape.width())
        }
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
        let size = self.shape.size().as_();
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

    /// Get the rectangle for the full size.
    pub fn rect(&self) -> Rect<usize, usize> {
        Rect::new(0, 0, self.shape.size().w, self.shape.size().h)
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

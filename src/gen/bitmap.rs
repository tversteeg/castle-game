use std::{
    fmt::{Debug, Formatter, Result},
    ops::{Bound, Index, IndexMut, RangeBounds},
};

use bitvec::{slice::BitSlice, vec::BitVec};
use vek::{Extent2, Rect, Vec2};

/// Binary 2D map.
#[derive(Clone, PartialEq)]
pub struct Bitmap {
    /// Size of the map.
    size: Extent2<usize>,
    /// Binary vector.
    map: BitVec,
}

impl Bitmap {
    /// Create a new empty map with a specified size.
    pub fn empty(size: Extent2<usize>) -> Self {
        let map = BitVec::repeat(false, size.product());

        Self { size, map }
    }

    /// Create from a bitvec.
    pub fn from_bitvec(map: BitVec, size: Extent2<usize>) -> Self {
        Self { size, map }
    }

    /// Apply a removal mask to a position.
    ///
    /// Returns a delta map of which pixels got updated the same size as the removal map.
    pub fn apply_removal_mask(&mut self, removal_mask: &Bitmap, offset: Vec2<usize>) -> Bitmap {
        puffin::profile_scope!("Apply removel mask");

        // Keep track of all pixels that got set
        let mut delta_map = Bitmap::empty(removal_mask.size);

        // Apply to the shape
        for y in 0..removal_mask.size.h {
            // Y start index on the removal delta map
            let delta_index = y * removal_mask.size.w;
            // Y start index on the target shape map
            let shape_index = (y + offset.y) * self.size.w;

            for x in 0..removal_mask.size.w {
                // PERF: use a bitwise operator and no loop here
                let delta_index = delta_index + x;
                if removal_mask[delta_index] {
                    let shape_index = shape_index + offset.x + x;
                    if self[shape_index] {
                        delta_map.set_at_index(delta_index, true);
                        self.set_at_index(shape_index, false);
                    }
                }
            }
        }

        delta_map
    }

    /// Virtually apply the offset and clip to fit a rectangle of `(0, 0, size.w, size.h)`.
    ///
    /// Returns the actual offset.
    pub fn clip(&self, offset: Vec2<i32>, size: Extent2<usize>) -> (Vec2<usize>, Self) {
        puffin::profile_scope!("Clip");

        if offset.x >= 0 && offset.y >= 0 {
            let total_size = size - offset.as_();
            if self.size.w <= total_size.w && self.size.h <= total_size.h {
                // Current rectangle fits in the big rectangle, no need to clip
                return (offset.as_(), self.clone());
            }
        }

        // Calculate the edges of the newly clipped rectangle
        let start_x = offset.x.max(0) as usize;
        let end_x = (offset.x + self.size.w as i32).clamp(0, size.w as i32) as usize;
        let start_y = offset.y.max(0) as usize;
        let end_y = (offset.y + self.size.h as i32).clamp(0, size.h as i32) as usize;

        let new_size = Extent2::new(end_x - start_x, end_y - start_y);
        let mut new_map = Self::empty(new_size);

        let index_start_x = if offset.x < 0 {
            (-offset.x) as usize
        } else {
            0
        };
        let index_start_y = if offset.y < 0 {
            (-offset.y) as usize
        } else {
            0
        };

        // Copy the old pixels
        for y in 0..new_size.h {
            let y_new_index = y * new_size.w;
            let y_cur_index = (index_start_y + y) * self.size.w;
            // PERF: speed this up with ranges
            for x in 0..new_size.w {
                let x_cur_index = index_start_x + x;
                let value = self[y_cur_index + x_cur_index];
                new_map.set_at_index(y_new_index + x, value);
            }
        }

        (Vec2::new(start_x, start_y), new_map)
    }

    /// Shrink to fit all pixels with padding.
    ///
    /// Returns the offset of the left top edge.
    pub fn shrink_with_padding(&mut self, padding: usize) -> Vec2<usize> {
        puffin::profile_scope!("Shrink with padding");

        let mut min = Vec2::new(usize::MAX, usize::MAX);
        let mut max: Vec2<usize> = Vec2::zero();

        // Find the edges of all filled pixels
        for y in 0..self.size.h {
            let y_index = y * self.size.w;
            for x in 0..self.size.w {
                if self[y_index + x] {
                    min.x = min.x.min(x);
                    min.y = min.y.min(y);
                    max.x = max.x.max(x);
                    max.y = max.y.max(y);
                }
            }
        }

        // Apply padding
        min.x = min.x.saturating_sub(padding);
        min.y = min.y.saturating_sub(padding);
        max.x = (max.x + padding + 1).min(self.size.w);
        max.y = (max.y + padding + 1).min(self.size.h);

        let neg_min = Vec2::new(-(min.x as i32), -(min.y as i32));

        // Clip by moving the rectangle negatively towards the minimum point
        *self = self
            .clip(neg_min, Extent2::new(max.x - min.x, max.y - min.y))
            .1;

        min
    }

    /// Perform a floodfill to zero out values where values were previously set.
    #[inline(always)]
    pub fn zeroing_floodfill(&mut self, position: Vec2<usize>) {
        puffin::profile_scope!("Floodfill");

        // Create a stack for pixels that need to be filled
        let mut stack = Vec::with_capacity(16);
        stack.push(position.x + position.y * self.width());

        while let Some(index) = stack.pop() {
            let x = index % self.width();
            let y = index / self.width();
            if x >= self.width() || y >= self.height() || !self[index] {
                continue;
            }

            // Fill the value
            self.set_at_index(index, false);

            // Push the neighbors
            // Right
            stack.push(index + 1);
            // Left
            stack.push(index.wrapping_sub(1));
            // Up
            stack.push(index + self.width());
            // Down
            stack.push(index.wrapping_sub(self.width()));
        }
    }

    /// Perform a floodfill to zero out values where values were previously set and fill another copy buffer.
    #[inline(always)]
    pub fn zeroing_floodfill_with_copy(&mut self, position: Vec2<usize>) -> Self {
        puffin::profile_scope!("Floodfill with copy");

        // Create a new empty buffer for the copy
        let mut copy = Self::empty(self.size());

        // Create a stack for pixels that need to be filled

        // Create a stack for pixels that need to be filled
        let mut stack = Vec::with_capacity(16);
        stack.push(position.x + position.y * self.width());

        while let Some(index) = stack.pop() {
            let x = index % self.width();
            let y = index / self.width();
            if x >= self.width() || y >= self.height() || !self[index] {
                continue;
            }

            // Fill the value
            self.set_at_index(index, false);
            copy.set_at_index(index, true);

            // Push the neighbors
            // Right
            stack.push(index + 1);
            // Left
            stack.push(index.wrapping_sub(1));
            // Up
            stack.push(index + self.width());
            // Down
            stack.push(index.wrapping_sub(self.width()));
        }

        copy
    }

    /// Set a pixel at coordinates.
    #[inline(always)]
    pub fn set<V>(&mut self, position: V, value: bool)
    where
        V: Into<Vec2<usize>>,
    {
        let position = position.into();
        let index = position.x + position.y * self.size.w;
        self.set_at_index(index, value);
    }

    /// Set a pixel at index of the map.
    #[inline(always)]
    pub fn set_at_index(&mut self, index: usize, value: bool) {
        debug_assert!(index < self.map.len());

        self.map.set(index, value);
    }

    /// Set pixels at index range.
    #[inline(always)]
    pub fn set_at_index_range<R>(&mut self, range: R, value: bool)
    where
        R: RangeBounds<usize>,
    {
        let start = match range.start_bound() {
            Bound::Included(start) => *start,
            Bound::Excluded(start) => start + 1,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(end) => *end,
            Bound::Excluded(end) => end - 1,
            Bound::Unbounded => self.map.len(),
        };

        self.map[start..end].fill(value);
    }

    /// Calculate whether the shape has multiple islands.
    ///
    /// This is an expensive calculation.
    pub fn has_multiple_islands(&self) -> bool {
        puffin::profile_scope!("Has multiple islands");

        // Do a floodfill on the first non-empty pixel found
        if let Some(filled_pixel) = self.first_one() {
            let mut check = self.clone();
            check.zeroing_floodfill(filled_pixel);

            // If any other pixel is still set it means there are multiple pixels
            !check.is_empty()
        } else {
            // Image is empty
            false
        }
    }

    /// Get the coordinates of the first non-zero pixel.
    #[inline(always)]
    pub fn first_one(&self) -> Option<Vec2<usize>> {
        let position_index = self.map.first_one()?;

        Some(Vec2::new(
            position_index % self.size.w,
            position_index / self.size.w,
        ))
    }

    // If any pixels are set.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.map.first_one().is_none()
    }

    /// Dimensions.
    #[inline(always)]
    pub fn size(&self) -> Extent2<usize> {
        self.size
    }

    /// Width of the map.
    #[inline(always)]
    pub fn width(&self) -> usize {
        self.size.w
    }

    /// Height of the map.
    #[inline(always)]
    pub fn height(&self) -> usize {
        self.size.h
    }

    /// Create a rect with an offset.
    #[inline(always)]
    pub fn rect(&self, offset: Vec2<usize>) -> Rect<usize, usize> {
        Rect::new(offset.x, offset.y, self.size.w, self.size.h)
    }
}

impl Index<usize> for Bitmap {
    type Output = bool;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        &self.map[index]
    }
}

impl Index<Vec2<usize>> for Bitmap {
    type Output = bool;

    #[inline(always)]
    fn index(&self, position: Vec2<usize>) -> &Self::Output {
        &self[position.x + position.y * self.size.w]
    }
}

impl Index<(usize, usize)> for Bitmap {
    type Output = bool;

    #[inline(always)]
    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        &self[x + y * self.size.w]
    }
}

impl Debug for Bitmap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Bitmap {}x{}:", self.size.w, self.size.h)?;

        for y in 0..self.size.h {
            for x in 0..self.size.w {
                write!(f, "{}", if self[(x, y)] { "X" } else { "." })?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

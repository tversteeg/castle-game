use std::{
    fmt::{Debug, Formatter, Result},
    ops::{Bound, Index, IndexMut, RangeBounds},
};

use bitvec::vec::BitVec;
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

        debug_assert!(offset.x + removal_mask.size.w <= self.size.w);
        debug_assert!(offset.y + removal_mask.size.h <= self.size.h);

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
            let mut total_size: Extent2<i32> = size.as_();
            total_size -= offset;
            if self.size.w <= total_size.w as usize && self.size.h <= total_size.h as usize {
                // Current rectangle fits in the big rectangle, no need to clip
                return (offset.as_(), self.clone());
            }
        }

        // Calculate the edges of the newly clipped rectangle
        let start_x = offset.x.max(0) as usize;
        let end_x = (offset.x + self.size.w as i32).clamp(0, size.w as i32) as usize;
        let start_y = offset.y.max(0) as usize;
        let end_y = (offset.y + self.size.h as i32).clamp(0, size.h as i32) as usize;

        let new_size = Extent2::new(end_x.saturating_sub(start_x), end_y.saturating_sub(start_y));
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

        // Copy the old pixels column by column
        for y in 0..new_size.h {
            new_map.copy_slice_from(
                Vec2::new(0, y),
                self,
                Vec2::new(index_start_x, index_start_y + y),
                new_size.w,
            )
        }

        debug_assert_eq!(new_map.map.len(), new_map.size.product());

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

        let new_size = Extent2::new(max.x - min.x + 1, max.y - min.y + 1);

        let previous = self.clone();

        // Create a new map to copy the old one into
        self.size = new_size + (padding * 2, padding * 2);
        self.map = BitVec::repeat(false, self.size.product());

        // Copy horizontal slices from the old one
        for y in 0..new_size.h {
            self.copy_slice_from(
                (padding, padding + y).into(),
                &previous,
                min + (0, y),
                new_size.w,
            )
        }

        debug_assert_eq!(self.map.len(), self.size.product());

        min - (padding, padding)
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
            if x < self.width() - 1 {
                stack.push(index + 1);
            }

            // Left
            if x > 0 {
                stack.push(index.wrapping_sub(1));
            }

            // Up
            if y < self.height() - 1 {
                stack.push(index + self.width());
            }

            // Down
            if y > 0 {
                stack.push(index.wrapping_sub(self.width()));
            }
        }
    }

    /// Perform a floodfill to zero out values where values were previously set and fill another copy buffer.
    #[inline(always)]
    pub fn zeroing_floodfill_with_copy(&mut self, position: Vec2<usize>) -> Self {
        puffin::profile_scope!("Floodfill with copy");

        // Create a new empty buffer for the copy
        let mut copy = Self::empty(self.size());

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
            if x < self.width() - 1 {
                stack.push(index + 1);
            }

            // Left
            if x > 0 {
                stack.push(index.wrapping_sub(1));
            }

            // Up
            if y < self.height() - 1 {
                stack.push(index + self.width());
            }

            // Down
            if y > 0 {
                stack.push(index.wrapping_sub(self.width()));
            }
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

    /// Copy a range from another image.
    #[inline(always)]
    pub fn copy_slice_from(
        &mut self,
        start: Vec2<usize>,
        other: &Self,
        other_start: Vec2<usize>,
        amount: usize,
    ) {
        debug_assert!(start.x + amount <= self.width());
        debug_assert!(other_start.x + amount <= other.width());

        let index = start.x + start.y * self.width();
        let other_index = other_start.x + other_start.y * other.width();

        self.map[index..(index + amount)]
            .copy_from_bitslice(&other.map[other_index..(other_index + amount)]);
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

#[cfg(test)]
mod tests {
    use bitvec::vec::BitVec;
    use vek::{Extent2, Vec2};

    use super::Bitmap;

    #[test]
    fn shrink_with_padding() {
        // 4x4 image filled with ones
        let size = Extent2::new(4, 4);
        let mut image = Bitmap::from_bitvec(BitVec::repeat(true, size.product()), size);
        // Set a pixel so we can check offset
        image.set((1, 1), false);

        let original = image.clone();

        image.shrink_with_padding(1);
        assert_eq!(image.size(), size + (2, 2), "{image:?}");
        assert!(!image[(2, 2)], "{image:?}");

        image.shrink_with_padding(2);
        assert_eq!(image.size(), size + (4, 4), "{image:?}");
        assert!(!image[(3, 3)], "{image:?}");

        image.shrink_with_padding(1);
        assert_eq!(image.size(), size + (2, 2), "{image:?}");
        assert!(!image[(2, 2)], "{image:?}");

        // When applying no padding the original should be set again
        image.shrink_with_padding(0);
        assert_eq!(image, original, "{image:?}");
        assert!(!image[(1, 1)], "{image:?}");
    }

    #[test]
    fn clip() {
        // 4x4 image filled with a single one
        let size = Extent2::new(4, 4);
        let mut image = Bitmap::empty(size);
        // Set a pixel so we can check offset
        image.set((1, 1), true);

        // Should clip first row of pixels
        let (new_offset, new_image) = image.clip(Vec2::new(-1, 0), size);
        assert_eq!(new_offset, Vec2::zero());
        assert_eq!(new_image.size(), size - (1, 0));
        assert!(new_image[(0, 1)]);

        // Should clip first column of pixels
        let (new_offset, new_image) = image.clip(Vec2::new(0, -1), size);
        assert_eq!(new_offset, Vec2::zero());
        assert_eq!(new_image.size(), size - (0, 1));
        assert!(new_image[(1, 0)]);

        // Shouldn't clip anything
        let (new_offset, new_image) = image.clip(Vec2::new(1, 1), size + (1, 1));
        assert_eq!(new_offset, Vec2::new(1, 1));
        assert_eq!(new_image.size(), size);
        assert!(new_image[(1, 1)]);

        // Shoul clip last row & column of pixels
        let (new_offset, new_image) = image.clip(Vec2::new(1, 1), size);
        assert_eq!(new_offset, Vec2::new(1, 1));
        assert_eq!(new_image.size(), size - (1, 1));
        assert!(new_image[(1, 1)]);

        // Shoul clip last row & column of pixels
        let (new_offset, new_image) = image.clip(Vec2::new(0, 0), size - (1, 1));
        assert_eq!(new_offset, Vec2::zero());
        assert_eq!(new_image.size(), size - (1, 1));
        assert!(new_image[(1, 1)]);
    }
}

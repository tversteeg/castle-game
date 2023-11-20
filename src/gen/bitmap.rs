use std::{
    fmt::{Debug, Formatter, Result, Write},
    ops::{Bound, Index, IndexMut, RangeBounds},
};

use bitvec::vec::BitVec;
use spiral::ChebyshevIterator;
use vek::{Extent2, Rect, Vec2};

use crate::gen::isoline::MarchingSquaresIterator;

use super::isoline::EdgeWalker;

/// How many debug characters to render horizontally in the terminal.
const HORIZONTAL_DEBUG_CHARACTERS: usize = 100;

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

        // Nothing to shrink since the map is empty
        debug_assert!(!self.is_empty());

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

        // Remove the padding, because it won't change the position
        min.x = min.x.saturating_sub(padding);
        min.y = min.y.saturating_sub(padding);

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
        // PERF: find a way to not make a full copy
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

    /// Try to get all continuously connected islands from the shape.
    pub fn islands(&self) -> Vec<Vec2<usize>> {
        puffin::profile_scope!("Try find islands");

        // Do a floodfill on the first non-empty pixel found
        // Check from the center instead of the start so the edges aren't checked
        let mut islands = Vec::new();

        // Copy the subsection so we can remove all pixels until it's empty
        let mut check = self.clone();

        // Check if any pixel hasn't been set yet
        while let Some(filled_pixel) = check.first_one_from_center() {
            // Floodfill from the pixel so it can be ignored
            check.zeroing_floodfill(filled_pixel);

            islands.push(filled_pixel);
        }

        islands
    }

    /// Calculate the area from a shape beginning at set position.
    ///
    /// Panics when starting position out of bounds or doesn't point to an edge of a shape.
    /// Also panics when shape edges can't be walked.
    pub fn area_from_shape_at_position(&self, shape_starting_position: Vec2<usize>) -> f64 {
        // If the starting point is on an empty pixel there's no shape thus no area
        if !self[shape_starting_position] {
            return 0.0;
        }

        EdgeWalker::new(shape_starting_position, self).walk_area()
    }

    /// Get the coordinates of the first non-zero pixel.
    #[inline(always)]
    pub fn first_one(&self) -> Option<Vec2<usize>> {
        let position_index = self.map.first_one()?;

        Some(Vec2::new(
            position_index % self.width(),
            position_index / self.width(),
        ))
    }

    /// Get the coordinates of the first non-zero pixel from the center of the image.
    #[inline(always)]
    pub fn first_one_from_center(&self) -> Option<Vec2<usize>> {
        self.next_or_previous_one(Vec2::new(self.size.w / 2, self.size.h / 2))
    }

    /// Find next non-zero value from position.
    #[inline(always)]
    pub fn next_or_previous_one(&self, position: Vec2<usize>) -> Option<Vec2<usize>> {
        puffin::profile_scope!("Next or previous non-zero pixel");

        debug_assert!(position.x < self.width());
        debug_assert!(position.y < self.height());

        ChebyshevIterator::new(
            position.x as i32,
            position.y as i32,
            (self.width().max(self.height()) / 2) as i32,
        )
        .find(|(x, y)| {
            *x >= 0
                && *x < self.width() as i32
                && *y >= 0
                && *y < self.height() as i32
                && self[(*x as usize, *y as usize)]
        })
        .map(|(x, y)| Vec2::new(x as usize, y as usize))
    }

    /// Calculate how many pixels got set.
    #[inline(always)]
    pub fn pixels_set(&self) -> usize {
        self.map.count_ones()
    }

    /// Create a debug string from the map, marking a specific position.
    #[cfg(feature = "debug")]
    pub fn debug_mark_position(&self, mark: Vec2<usize>) -> String {
        let mut canvas = drawille::Canvas::new(self.width() as u32, self.height() as u32);
        let mut debug = String::new();

        writeln!(&mut debug, "Bitmap {}x{}:", self.size.w, self.size.h).unwrap();

        for y in 0..self.size.h {
            for x in 0..self.size.w {
                if self[(x, y)] {
                    canvas.set(x as u32, y as u32);
                }
            }
        }

        // TODO: fix for small images
        const X_SIZE: usize = 20;
        if mark.x < self.width() && mark.y < self.height() {
            canvas.line_colored(
                mark.x.wrapping_sub(X_SIZE).clamp(0, self.width()) as u32,
                mark.y.wrapping_sub(X_SIZE).clamp(0, self.height()) as u32,
                mark.x.wrapping_add(X_SIZE).clamp(0, self.width()) as u32,
                mark.y.wrapping_add(X_SIZE).clamp(0, self.height()) as u32,
                drawille::PixelColor::Red,
            );
            canvas.line_colored(
                mark.x.wrapping_sub(X_SIZE).clamp(0, self.width()) as u32,
                mark.y.wrapping_add(X_SIZE).clamp(0, self.height()) as u32,
                mark.x.wrapping_add(X_SIZE).clamp(0, self.width()) as u32,
                mark.y.wrapping_sub(X_SIZE).clamp(0, self.height()) as u32,
                drawille::PixelColor::Red,
            );
        }

        debug.push_str(&canvas.frame());

        debug
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

#[cfg(feature = "debug")]
impl Debug for Bitmap {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(
            f,
            "{}",
            self.debug_mark_position(Vec2::new(usize::MAX, usize::MAX))
        )
    }
}

#[cfg(test)]
mod tests {
    use bitvec::vec::BitVec;
    use vek::{Extent2, Vec2};

    use super::Bitmap;

    #[cfg(feature = "debug")]
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

    #[test]
    fn floodfill() {
        // 6x6 image filled with ones
        let size = Extent2::new(6, 6);
        let mut image = Bitmap::from_bitvec(BitVec::repeat(true, size.product()), size);

        // Create a diagonal of unset values
        for i in 0..size.w {
            image.set((size.w - i - 1, i), false);
        }

        // Performa floodfill on the top part
        let removed = image.zeroing_floodfill_with_copy(Vec2::new(1, 1));

        // Parts should be equal but rotated
        assert!(removed[(0, 0)]);
        assert!(image[(size.w - 1, size.h - 1)]);
        assert_eq!(removed.pixels_set(), image.pixels_set());
    }
}

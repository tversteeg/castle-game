use std::collections::VecDeque;

use itertools::Itertools;
use vek::{Aabr, Extent2, Rect, Vec2};

use crate::physics::collision::shape::Shape;

use super::bitmap::Bitmap;

/// Isoline mesh from a bitmap that can be updated.
#[derive(Debug, Default)]
pub struct Isoline {
    /// List of vertices connecting into a line for this mesh.
    vertices: Vec<Vec2<f64>>,
}

impl Isoline {
    /// Generate from a bitmap.
    ///
    /// Bitmap is not allowed to contain multiple non-connected pixels.
    #[must_use]
    pub fn from_bitmap(bitmap: &Bitmap) -> Self {
        // Create the vertices with a marching squares iterator over the bitmap
        let vertices = MarchingSquaresIterator::new_find_starting_point(bitmap, [])
            .map(Option::unwrap)
            .map(Vec2::as_)
            .collect::<Vec<_>>();

        // Simplify the segments
        let vertices = crate::gen::rdp::ramer_douglas_peucker(&vertices, 1.0);

        Self { vertices }
    }

    /// Update a region on the bitmap.
    ///
    /// This is an optimization so the whole shape doesn't have to be recalculated.
    ///
    /// Assumes no islands exist on the bitmap.
    /// If the whole shape is cleared an extra border of 1 pixel should be added to each side.
    pub fn update(&mut self, bitmap: &Bitmap, delta_mask: &Bitmap, mask_position: Vec2<usize>) {
        puffin::profile_scope!("Update isoline");

        // PERF: don't do a full recalculation
        let vertices = MarchingSquaresIterator::new_find_starting_point(bitmap, [])
            .map(Option::unwrap)
            .map(Vec2::as_)
            .collect::<Vec<_>>();

        // Simplify the segments
        self.vertices = crate::gen::rdp::ramer_douglas_peucker(&vertices, 1.0);

        /*
        // Insert the newly generated vertices
        // PERF: find a way to do this in a single call
        for vert in delta_mask_vertices.into_iter().map(Vec2::as_) {
            self.vertices.insert(first_index, vert);
        }
        */
    }

    /// Create a collider from the vertices.
    #[must_use]
    pub fn to_collider(&self, size: Extent2<f64>) -> Shape {
        puffin::profile_scope!("Isoline to collider");

        Shape::linestrip(
            &self
                .vertices
                .iter()
                .map(|vert| *vert - (size.w / 2.0, 0.0))
                .collect::<Vec<_>>(),
        )
    }

    /// Find any vertices laying on the removal mask.
    fn vertices_on_mask(
        &self,
        delta_mask: &Bitmap,
        mask_position: Vec2<usize>,
    ) -> Vec<(usize, Vec2<usize>)> {
        puffin::profile_scope!("Find vertices on mask");

        let mask_region = delta_mask.rect(mask_position);

        self.vertices
            .iter()
            .enumerate()
            // Convert vert to usize
            .map(|(index, vert)| (index, vert.as_()))
            // Do a simple pass filtering all vertices not in the region out
            .filter(|(_index, vert)| mask_region.contains_point(*vert))
            // Do a pass filtering all vertices not on the removal mask out
            .filter(|(_index, vert)| {
                let vert_index_on_mask =
                    (vert.x - mask_region.x) + (vert.y - mask_region.y) * mask_region.w;

                delta_mask[vert_index_on_mask]
            })
            .collect()
    }
}

/// Marching square walker over the source image.
#[derive(Debug, Clone)]
struct MarchingSquaresIterator<'a, const STOP_COUNT: usize> {
    /// Current position.
    pos: Vec2<usize>,
    /// Starting position.
    start: Vec2<usize>,
    /// Iterator fails when reaching any of these coordinates.
    stop_at: [Vec2<usize>; STOP_COUNT],
    /// We are done iterating.
    done: bool,
    /// Previous direction.
    ///
    /// Allows us to cross ambiguous points.
    prev_dir: MarchingSquaresWalkerDirection,
    /// Image we are walking over.
    map: &'a Bitmap,
}

impl<'a, const STOP_COUNT: usize> MarchingSquaresIterator<'a, STOP_COUNT> {
    /// Create a new iterator walking over the source image using the marching square algorithm.
    ///
    /// <div class='warning'>BitMap must be padded by 0 bits around the edges!</div>
    pub fn new(
        starting_position: Vec2<usize>,
        map: &'a Bitmap,
        stop_at: [Vec2<usize>; STOP_COUNT],
    ) -> Self {
        let pos = starting_position;
        let start = pos;
        // Initial value doesn't matter
        let prev_dir = MarchingSquaresWalkerDirection::Up;
        let done = false;

        Self {
            done,
            pos,
            start,
            prev_dir,
            stop_at,
            map,
        }
    }

    /// Create a new iterator walking over the source image using the marching square algorithm.
    ///
    /// The starting point is found as the first bit that's set.
    ///
    /// <div class='warning'>BitMap must be padded by 0 bits around the edges!</div>
    pub fn new_find_starting_point(map: &'a Bitmap, stop_at: [Vec2<usize>; STOP_COUNT]) -> Self {
        let starting_position = map
            .first_one()
            .expect("Cannot create marching squares iterator over empty map");

        Self::new(starting_position, map, stop_at)
    }

    /// Convert a position to a 4bit number looking at it as a 2x2 grid if possible.
    #[inline(always)]
    fn dir_number(&self) -> u8 {
        // Ensure we don't go out of bounds
        debug_assert!(self.pos.x > 0);
        debug_assert!(self.pos.y > 0);

        let index = self.pos.x + self.pos.y * self.map.width();
        let topleft = self.map[index - 1 - self.map.width()] as u8;
        let topright = self.map[index - self.map.width()] as u8;
        let botleft = self.map[index - 1] as u8;
        let botright = self.map[index] as u8;

        (botright << 3) | (botleft << 2) | (topright << 1) | topleft
    }

    /// Whether the current position direction number is any of the passed direction combinations.
    #[inline(always)]
    fn is_dir_number(&self, numbers: [u8; 3]) -> bool {
        let dir_number = self.dir_number();

        dir_number == numbers[0] || dir_number == numbers[1] || dir_number == numbers[2]
    }

    /// Check whether the coordinate is reached.
    #[inline(always)]
    fn should_stop(&mut self) -> bool {
        for stop_at in self.stop_at {
            if self.pos == stop_at {
                self.done = true;

                return true;
            }
        }

        false
    }
}

impl<'a, const STOP_COUNT: usize> Iterator for MarchingSquaresIterator<'a, STOP_COUNT> {
    type Item = Option<Vec2<usize>>;

    fn next(&mut self) -> Option<Self::Item> {
        puffin::profile_scope!("Marching squares iterator step");

        if self.done {
            return None;
        }

        // Move the cursor based on the edge direction, following the outline
        // PERF: we are doing an extra check that's unused whenever the direction changes, find a way to improve this
        match self.dir_number() {
            // Up
            1 | 5 | 13 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.y -= 1;
                    debug_assert!(self.pos.y > 0);
                    if self.should_stop() {
                        return Some(None);
                    }

                    if !self.is_dir_number([1, 5, 13]) {
                        break;
                    }
                }

                self.prev_dir = MarchingSquaresWalkerDirection::Up;
            }
            // Down
            8 | 10 | 11 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.y += 1;
                    debug_assert!(self.pos.y < self.map.height());
                    if self.should_stop() {
                        return Some(None);
                    }

                    if !self.is_dir_number([8, 10, 11]) {
                        break;
                    }
                }

                self.prev_dir = MarchingSquaresWalkerDirection::Down;
            }
            // Left
            4 | 12 | 14 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.x -= 1;
                    debug_assert!(self.pos.x > 0);
                    if self.should_stop() {
                        return Some(None);
                    }

                    if !self.is_dir_number([4, 12, 14]) {
                        break;
                    }
                }

                self.prev_dir = MarchingSquaresWalkerDirection::Left;
            }
            // Right
            2 | 3 | 7 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.x += 1;
                    debug_assert!(self.pos.x < self.map.width());
                    if self.should_stop() {
                        return Some(None);
                    }

                    if !self.is_dir_number([2, 3, 7]) {
                        break;
                    }
                }

                self.prev_dir = MarchingSquaresWalkerDirection::Right;
            }
            // Down if previous was left, up if previous was right
            9 => {
                if self.prev_dir == MarchingSquaresWalkerDirection::Left {
                    self.pos.y += 1;
                } else {
                    self.pos.y -= 1;
                }

                if self.should_stop() {
                    return Some(None);
                }
            }
            // Right if previous was down, left if previous was up
            6 => {
                if self.prev_dir == MarchingSquaresWalkerDirection::Down {
                    self.pos.x += 1;
                } else {
                    self.pos.x -= 1;
                }

                if self.should_stop() {
                    return Some(None);
                }
            }
            _ => panic!("Unknown direction"),
        }

        if self.pos == self.start {
            // We made a full loop
            self.done = true;
        }

        Some(Some(self.pos))
    }
}

/// Directions the walker can go to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarchingSquaresWalkerDirection {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;
    use vek::{Extent2, Vec2};

    use super::MarchingSquaresIterator;

    #[test]
    fn test_marching_cubes_iterator() {
        #[rustfmt::skip]
        let image: &BitSlice = bits![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
            0, 1, 1, 1, 1, 1, 0,
            0, 1, 1, 1, 1, 1, 0,
            0, 0, 1, 1, 1, 1, 0,
            0, 0, 1, 0, 1, 1, 0,
            0, 0, 0, 1, 1, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ];

        let outline: Vec<_> =
            MarchingSquaresIterator::new(Vec2::new(2, 2), image, Extent2::new(7, 9)).collect();
        assert_eq!(outline.len(), 19);
    }
}

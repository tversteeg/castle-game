use std::collections::VecDeque;

use bitvec::slice::BitSlice;
use itertools::Itertools;
use vek::{Aabr, Extent2, Rect, Vec2};

use crate::physics::collision::shape::Shape;

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
    pub fn from_bitmap(bitmap: &BitSlice, size: Extent2<usize>) -> Self {
        // Create the vertices with a marching squares iterator over the bitmap
        let vertices = MarchingSquaresIterator::new_find_starting_point(bitmap, size, [])
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
    pub fn update(
        &mut self,
        bitmap: &BitSlice,
        size: Extent2<usize>,
        removal_mask: &BitSlice,
        mask_region: Rect<usize, usize>,
    ) {
        puffin::profile_scope!("Update isoline");

        debug_assert_eq!(bitmap.len(), size.w * size.h);
        debug_assert_eq!(removal_mask.len(), mask_region.w * mask_region.h);

        // Find any vertices laying on the removal mask
        let vertices_on_mask = self
            .vertices
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

                removal_mask.get(vert_index_on_mask).is_some()
            })
            .collect::<Vec<_>>();

        if vertices_on_mask.is_empty() {
            // TODO: what do we do when the line is longer than the area?
            println!("Collider update skipped");

            return;
        }

        // Check if we have multiple line parts in the removal mask
        if vertices_on_mask
            .iter()
            .zip(vertices_on_mask.iter().skip(1))
            .any(|((cur_index, _), (next_index, _))| *cur_index != next_index - 1)
        {
            // TODO: handle multiple lines
            todo!()
        }

        // Create vertices with a marching squares iterator over the mask bitmap
        let mut removal_mask_vertices = MarchingSquaresIterator::new_find_starting_point(
            removal_mask,
            mask_region.extent(),
            [],
        )
        .map(Option::unwrap)
        // Put them in the coordinates of the main bitmap again
        .map(|relative_vert| relative_vert + mask_region.position())
        .collect::<VecDeque<_>>();
        assert!(!removal_mask_vertices.is_empty());

        // Get the first vertex so we can shift the removal mask vertices to the nearest item
        let (first_removal_index, _) = vertices_on_mask.first().unwrap();
        let first_index = if let Some(index) = first_removal_index.checked_sub(1) {
            index
        } else {
            // Index was 0, wrap around
            self.vertices.len() - 1
        };
        let first_pos: Vec2<usize> = self.vertices.get(first_index).unwrap().as_();

        // Find the nearest to the first old vertex
        let (closest, _) = removal_mask_vertices
            .iter()
            .enumerate()
            .min_by_key(|(_index, vert)| vert.x * first_pos.x + vert.y * first_pos.y)
            // Safe because we already check if it's empty
            .unwrap();

        // Put the removal vertices list in proper order
        removal_mask_vertices.rotate_left(closest);

        dbg!(&closest);

        // Remove the vertices
        for (index, _vert) in vertices_on_mask.into_iter().rev() {
            self.vertices.remove(index);
        }

        // Insert the newly generated vertices
        // PERF: find a way to do this in a single call
        for vert in removal_mask_vertices.into_iter().map(Vec2::as_) {
            self.vertices.insert(first_index, vert);
        }
    }

    /// Create a collider from the vertices.
    #[must_use]
    pub fn to_collider(&self, size: Extent2<f64>) -> Shape {
        Shape::linestrip(
            &self
                .vertices
                .iter()
                .map(|vert| *vert - (size.w / 2.0, 0.0))
                .collect::<Vec<_>>(),
        )
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
    map: &'a BitSlice,
    /// Size of the image.
    size: Extent2<usize>,
}

impl<'a, const STOP_COUNT: usize> MarchingSquaresIterator<'a, STOP_COUNT> {
    /// Create a new iterator walking over the source image using the marching square algorithm.
    ///
    /// <div class='warning'>BitMap must be padded by 0 bits around the edges!</div>
    pub fn new(
        starting_position: Vec2<usize>,
        map: &'a BitSlice,
        size: Extent2<usize>,
        stop_at: [Vec2<usize>; STOP_COUNT],
    ) -> Self {
        debug_assert_eq!(map.len(), size.w * size.h);

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
            size,
        }
    }

    /// Create a new iterator walking over the source image using the marching square algorithm.
    ///
    /// The starting point is found as the first bit that's set.
    ///
    /// <div class='warning'>BitMap must be padded by 0 bits around the edges!</div>
    pub fn new_find_starting_point(
        map: &'a BitSlice,
        size: Extent2<usize>,
        stop_at: [Vec2<usize>; STOP_COUNT],
    ) -> Self {
        let starting_position_index = map
            .first_one()
            .expect("Cannot create marching squares iterator over empty map");
        let starting_position = Vec2::new(
            starting_position_index % size.w,
            starting_position_index / size.w,
        );
        debug_assert_eq!(
            starting_position.x + starting_position.y * size.w,
            starting_position_index
        );

        Self::new(starting_position, map, size, stop_at)
    }

    /// Convert a position to a 4bit number looking at it as a 2x2 grid if possible.
    #[inline(always)]
    fn dir_number(&self) -> u8 {
        // Ensure we don't go out of bounds
        debug_assert!(self.pos.x > 0);
        debug_assert!(self.pos.y > 0);

        let index = self.pos.x + self.pos.y * self.size.w;
        let topleft = self.map[index - 1 - self.size.w] as u8;
        let topright = self.map[index - self.size.w] as u8;
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
                    debug_assert!(self.pos.y < self.size.h);
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
                    debug_assert!(self.pos.x < self.size.w);
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
            None
        } else {
            Some(Some(self.pos))
        }
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

use bitvec::slice::BitSlice;
use itertools::Itertools;
use vek::{Aabr, Extent2, Vec2};

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
    /// Can return multiple items for each island.
    #[must_use]
    pub fn from_bitmap(bitmap: &BitSlice, size: Extent2<usize>) -> Vec<Self> {
        let vertices = Vec::new();

        // Construct one main element that can be updated
        let mut this = Self { vertices };

        // "Update" for the whole image
        let min = Vec2::zero();
        let max = Vec2::new(size.w, size.h);
        let mut islands = this.update(bitmap, size, Aabr { min, max }.as_());

        // Push the main item so everything is bundled
        islands.push(this);

        islands
    }

    /// Update a region on the bitmap.
    ///
    /// This is an optimization so the whole shape doesn't have to be recalculated.
    ///
    /// When a new island is formed it's returned.
    #[must_use]
    pub fn update(
        &mut self,
        bitmap: &BitSlice,
        size: Extent2<usize>,
        region: Aabr<usize>,
    ) -> Vec<Self> {
        puffin::profile_scope!("Update isoline");
        // TODO: detect islands
        // TODO: use region

        debug_assert!(bitmap.len() >= size.w * size.h);
        debug_assert!(region.max.x <= size.w);
        debug_assert!(region.max.y <= size.h);

        // Create the vertices with a marching squares iterator over the bitmap
        let vertices = MarchingSquaresIterator::new_find_starting_point(bitmap, size, [])
            .map(Option::unwrap)
            .collect::<Vec<_>>();

        // Get all edges at the corner of the image
        let min = region.min;
        let max = region.max - (1, 1);
        let top_edge = (min.x..max.x).map(|x| Vec2::new(x, min.y));
        let bot_edge = (min.x..max.x).map(|x| Vec2::new(x, max.y));
        let left_edge = (min.y..max.y).map(|y| Vec2::new(min.x, y));
        let right_edge = (min.y..max.y).map(|y| Vec2::new(max.x, y));

        // Find all edges for the rectangle and check if they are new islands
        let new_shapes = top_edge
            .chain(bot_edge)
            .filter(|edge_vert| {
                // Check if the aligned pixel is different so it's a line
                let index = edge_vert.x + edge_vert.y * size.w;
                bitmap[index] != bitmap[index + 1]
            })
            .chain(left_edge.chain(right_edge).filter(|edge_vert| {
                // Check if the aligned pixel is different so it's a line
                let index = edge_vert.x + edge_vert.y * size.w;
                bitmap[index] != bitmap[index + size.w]
            }))
            .filter_map(|vert| {
                // Detect a new shape
                let vertices =
                    MarchingSquaresIterator::new_find_starting_point(bitmap, size, [vert])
                        .collect::<Vec<_>>();

                if !vertices.contains(&None) {
                    dbg!(vertices.len());

                    // New island shape detected
                    let vertices = vertices
                        .into_iter()
                        .map(Option::unwrap)
                        .map(Vec2::as_)
                        .collect::<Vec<_>>();

                    // Simplify the segments
                    let vertices = crate::gen::rdp::ramer_douglas_peucker(&vertices, 1.0);

                    Some(Self { vertices })
                } else {
                    None
                }
            })
            .collect();

        // Simplify the segments
        self.vertices = crate::gen::rdp::ramer_douglas_peucker(
            &vertices.into_iter().map(Vec2::as_).collect::<Vec<_>>(),
            1.0,
        );

        new_shapes
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

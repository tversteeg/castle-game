use vek::{Extent2, Vec2};

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
        let vertices = MarchingSquaresIterator::new_find_starting_point(bitmap)
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
    pub fn update(&mut self, bitmap: &Bitmap, _delta_mask: &Bitmap, _mask_position: Vec2<usize>) {
        puffin::profile_scope!("Update isoline");

        // PERF: don't do a full recalculation
        let vertices = MarchingSquaresIterator::new_find_starting_point(bitmap)
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
    pub fn to_collider(&self) -> Shape {
        puffin::profile_scope!("Isoline to collider");

        Shape::linestrip(&self.vertices)
    }
}

/// Marching square walker over the source image.
#[derive(Debug, Clone)]
pub struct MarchingSquaresIterator<'a> {
    /// Edge walker main algorithm.
    edge_walker: EdgeWalker<'a>,
    /// Starting position.
    start: Vec2<usize>,
    /// We are done iterating.
    done: bool,
}

impl<'a> MarchingSquaresIterator<'a> {
    /// Create a new iterator walking over the source image using the marching square algorithm.
    ///
    /// <div class='warning'>BitMap must be padded by 0 bits around the edges!</div>
    pub fn new(starting_position: Vec2<usize>, map: &'a Bitmap) -> Self {
        let edge_walker = EdgeWalker::new(starting_position, map);
        let start = starting_position;
        let done = false;

        Self {
            edge_walker,
            start,
            done,
        }
    }

    /// Create a new iterator walking over the source image using the marching square algorithm.
    ///
    /// The starting point is found as the first bit that's set.
    ///
    /// <div class='warning'>BitMap must be padded by 0 bits around the edges!</div>
    pub fn new_find_starting_point(map: &'a Bitmap) -> Self {
        let starting_position = map
            .first_one()
            .expect("Cannot create marching squares iterator over empty map");

        Self::new(starting_position, map)
    }
}

impl<'a> Iterator for MarchingSquaresIterator<'a> {
    type Item = Vec2<usize>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        puffin::profile_scope!("Marching squares iterator step");

        if self.done {
            return None;
        }

        // Generate new coordinates
        self.edge_walker.step();

        if self.edge_walker.position() == self.start {
            // We made a full loop
            self.done = true;
        }

        Some(self.edge_walker.position())
    }
}

/// Walk around the edge of a bitmap.
///
/// Basis of [`MarchingSquaresIterator`].
#[derive(Debug, Clone)]
pub struct EdgeWalker<'a> {
    /// Current position.
    pos: Vec2<usize>,
    /// Previous direction.
    ///
    /// Allows us to cross ambiguous points.
    prev_dir: EdgeWalkerDirection,
    /// Image we are walking over.
    map: &'a Bitmap,
}

impl<'a> EdgeWalker<'a> {
    /// New edge walker at position.
    pub fn new(pos: Vec2<usize>, map: &'a Bitmap) -> Self {
        let prev_dir = EdgeWalkerDirection::Up;

        Self { pos, prev_dir, map }
    }

    /// Do a single step, not skipping any pixels.
    #[inline]
    pub fn single_step(&mut self) {
        puffin::profile_scope!("Edge walker single step");

        // Move the cursor based on the edge direction, following the outline
        match self.dir_number() {
            // Up
            1 | 5 | 13 => {
                self.pos.y -= 1;
                debug_assert!(self.pos.y > 0);

                self.prev_dir = EdgeWalkerDirection::Up;
            }
            // Down
            8 | 10 | 11 => {
                self.pos.y += 1;
                debug_assert!(self.pos.y < self.map.height());

                self.prev_dir = EdgeWalkerDirection::Down;
            }
            // Left
            4 | 12 | 14 => {
                self.pos.x -= 1;
                debug_assert!(self.pos.x > 0);

                self.prev_dir = EdgeWalkerDirection::Left;
            }
            // Right
            2 | 3 | 7 => {
                self.pos.x += 1;
                debug_assert!(self.pos.x < self.map.width());

                self.prev_dir = EdgeWalkerDirection::Right;
            }
            // Down if previous was left, up if previous was right
            9 => {
                if self.prev_dir == EdgeWalkerDirection::Left {
                    self.pos.y += 1;
                } else {
                    self.pos.y -= 1;
                }
            }
            // Right if previous was down, left if previous was up
            6 => {
                if self.prev_dir == EdgeWalkerDirection::Down {
                    self.pos.x += 1;
                } else {
                    self.pos.x -= 1;
                }
            }
            _ => panic!("Unknown direction"),
        }
    }

    /// Do a step, skipping any pixels in the same direction.
    #[inline]
    pub fn step(&mut self) {
        puffin::profile_scope!("Edge walker step");

        // Move the cursor based on the edge direction, following the outline
        match self.dir_number() {
            // Up
            1 | 5 | 13 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.y -= 1;
                    debug_assert!(self.pos.y > 0);

                    if !self.is_dir_number([1, 5, 13]) {
                        break;
                    }
                }

                self.prev_dir = EdgeWalkerDirection::Up;
            }
            // Down
            8 | 10 | 11 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.y += 1;
                    debug_assert!(self.pos.y < self.map.height());

                    if !self.is_dir_number([8, 10, 11]) {
                        break;
                    }
                }

                self.prev_dir = EdgeWalkerDirection::Down;
            }
            // Left
            4 | 12 | 14 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.x -= 1;
                    debug_assert!(self.pos.x > 0);

                    if !self.is_dir_number([4, 12, 14]) {
                        break;
                    }
                }

                self.prev_dir = EdgeWalkerDirection::Left;
            }
            // Right
            2 | 3 | 7 => {
                // Keep walking, ignoring all parts between the line segments
                loop {
                    self.pos.x += 1;
                    debug_assert!(self.pos.x < self.map.width());

                    if !self.is_dir_number([2, 3, 7]) {
                        break;
                    }
                }

                self.prev_dir = EdgeWalkerDirection::Right;
            }
            // Down if previous was left, up if previous was right
            9 => {
                if self.prev_dir == EdgeWalkerDirection::Left {
                    self.pos.y += 1;
                } else {
                    self.pos.y -= 1;
                }
            }
            // Right if previous was down, left if previous was up
            6 => {
                if self.prev_dir == EdgeWalkerDirection::Down {
                    self.pos.x += 1;
                } else {
                    self.pos.x -= 1;
                }
            }
            _ => panic!("Unknown direction"),
        }
    }

    /// Current position of the walker.
    #[inline(always)]
    pub fn position(&self) -> Vec2<usize> {
        self.pos
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
}

/// Directions the walker can go to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeWalkerDirection {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;
    use vek::{Extent2, Vec2};

    use crate::gen::bitmap::Bitmap;

    use super::MarchingSquaresIterator;

    #[test]
    fn test_marching_cubes_iterator() {
        #[rustfmt::skip]
        let image  = Bitmap::from_bitvec(bits![
            0, 0, 0, 0, 0, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
            0, 1, 1, 1, 1, 1, 0,
            0, 1, 1, 1, 1, 1, 0,
            0, 0, 1, 1, 1, 1, 0,
            0, 0, 1, 0, 1, 1, 0,
            0, 0, 0, 1, 1, 0, 0,
            0, 0, 1, 1, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0,
        ].to_bitvec(), Extent2::new(7, 9));

        let outline: Vec<_> = MarchingSquaresIterator::new(Vec2::new(2, 2), &image).collect();
        assert_eq!(outline.len(), 20);
    }
}

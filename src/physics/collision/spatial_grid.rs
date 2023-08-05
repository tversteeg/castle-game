use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
};

use arrayvec::ArrayVec;
use itertools::Itertools;
use vek::{Aabr, Extent2, Vec2};

/// Spatial hash grid with fixed buckets divided over an area so potential collision pairs can be found quickly.
///
/// Entities gets dropped when added to buckets that are already full.
///
/// Because of not allowing arithmetic (yet) in Rust const generics the following needs to be calculated:
/// - `SIZE` is `(WIDTH / STEP * HEIGHT / STEP) as usize`.
/// - `STEP` is how many pixels fit in each bucket, `WIDTH % STEP` and `HEIGHT % STEP` must both be zero
/// - `BUCKET` is amount of simultaneous objects can be checked at the same time
/// - `I` is the type for identifying another object. It's smart to keep this as small as possible
pub struct SpatialGrid<
    I,
    const WIDTH: u16,
    const HEIGHT: u16,
    const STEP: u16,
    const BUCKET: usize,
    const SIZE: usize,
> where
    I: Debug + Copy + Eq + Hash,
{
    /// Buckets spread out over the grid.
    buckets: [ArrayVec<I, BUCKET>; SIZE],
}

impl<
        I,
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    > SpatialGrid<I, WIDTH, HEIGHT, STEP, BUCKET, SIZE>
where
    I: Debug + Copy + Eq + Hash,
{
    const STEPPED_WIDTH: u16 = WIDTH / STEP;
    const STEPPED_HEIGHT: u16 = HEIGHT / STEP;

    /// Construct a new grid.
    // TODO: wait for either `.map` or `std::array::from_fn` to become const generic to make this const generic.
    pub fn new() -> Self {
        puffin::profile_function!();

        assert!(STEP >= 4);
        assert!(
            WIDTH % STEP == 0,
            "WIDTH {WIDTH} is not dividable by STEP {STEP}"
        );
        assert!(
            HEIGHT % STEP == 0,
            "HEIGHT {HEIGHT} is not dividable by STEP {STEP}"
        );
        assert_eq!(SIZE, (Self::STEPPED_WIDTH * Self::STEPPED_HEIGHT) as usize);

        let buckets = std::array::from_fn(|_| ArrayVec::new_const());

        Self { buckets }
    }

    /// Flush all buckets returning an iterator of all matching pairs.
    ///
    /// The list of matching pairs doesn't contain the same pairs twice.
    pub fn flush(&mut self) -> impl Iterator<Item = (I, I)> {
        puffin::profile_function!();

        // Resulting unique pairs
        let mut pairs = HashSet::new();

        for bucket in self.buckets.iter_mut() {
            // Combine all items in the bucket
            bucket
                // Remove everything from the bucket
                .take()
                .into_iter()
                // Get all possible combinations of values in the bucket as tuples
                .tuple_combinations()
                // We don't have to check the order of the pair because the order of entry is guaranteed to be the same for earlier intersections
                .for_each(|pair| {
                    // Due to the nature of the hash function we also don't have to check whether it's already added or not
                    pairs.insert(pair);
                });
        }

        pairs.into_iter()
    }

    /// Store an entity AABR rectangle.
    ///
    /// This will fill all buckets that are colliding with this rectangle.
    ///
    /// Drops an entity when the bucket is full.
    pub fn store_aabr(&mut self, aabr: Aabr<u16>, id: I) {
        puffin::profile_function!();

        let edge = Vec2::new(Self::STEPPED_WIDTH - 1, Self::STEPPED_HEIGHT - 1);
        let start = Vec2::<u16>::min(aabr.min / STEP, edge);
        let end = Vec2::<u16>::min(aabr.max / STEP, edge);

        for y in start.y..=end.y {
            for x in start.x..=end.x {
                self.add_to_bucket(x + y * Self::STEPPED_WIDTH, id);
            }
        }
    }

    /// Add entity to bucket at index.
    fn add_to_bucket(&mut self, index: u16, id: I) {
        let bucket = self
            .buckets
            .get_mut(index as usize)
            .expect("Entity out of range");

        // When the bucket is overflowing drop the entity
        if bucket.is_full() {
            return;
        }

        // SAFETY: we can push safely because we already checked if it's full
        unsafe {
            bucket.push_unchecked(id);
        }
    }

    /// Convert a coordinate to a bucket index coordinate.
    fn coordinate_to_index(coord: Vec2<u16>) -> u16 {
        // Divide by step size
        let x = coord.x / STEP;
        let y = coord.y / STEP;
        debug_assert!(x < Self::STEPPED_WIDTH);
        debug_assert!(y < Self::STEPPED_HEIGHT);

        // This shouldn't overflow since the coordinates have been divided by the step size
        x + y * Self::STEPPED_WIDTH
    }
}

impl<
        I,
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    > Display for SpatialGrid<I, WIDTH, HEIGHT, STEP, BUCKET, SIZE>
where
    I: Debug + Copy + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Draw a grid of how many items are in the buckets
        for (index, bucket) in self.buckets.iter().enumerate() {
            if index % Self::STEPPED_WIDTH as usize == 0 {
                writeln!(f)?;
            }

            if bucket.is_empty() {
                write!(f, " . ")?;
            } else {
                write!(f, "{:^3}", bucket.len())?;
            }
        }

        Ok(())
    }
}

impl<
        I,
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    > Debug for SpatialGrid<I, WIDTH, HEIGHT, STEP, BUCKET, SIZE>
where
    I: Debug + Copy + Eq + Hash,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "self")
    }
}

#[cfg(test)]
mod tests {
    use vek::{Extent2, Vec2};

    use super::SpatialGrid;

    /// Test different grid constructions.
    #[test]
    fn test_insertion() {
        const WIDTH: u16 = 32;
        const HEIGHT: u16 = 32;
        const STEP: u16 = 16;
        const BUCKET: usize = 3;
        const SIZE: usize = (WIDTH / STEP * HEIGHT / STEP) as usize;

        let mut grid = SpatialGrid::<u8, WIDTH, HEIGHT, STEP, BUCKET, SIZE>::new();

        // Store 2 entities in the same bucket, and 1 in a different one
        grid.store_entity(Vec2::new(30, 30), 0);
        grid.store_entity(Vec2::new(31, 30), 1);
        grid.store_entity(Vec2::new(0, 0), 2);

        // Get the entities back as a pair by flushing all buckets
        let pairs = grid.flush().collect::<Vec<_>>();
        assert_eq!(pairs, [(0, 1)]);

        // Store 3 entities in the same bucket, the order here matters for which one is left of the tuple and which one right
        grid.store_entity(Vec2::new(16, 16), 0);
        grid.store_entity(Vec2::new(16 + 15, 16), 1);
        grid.store_entity(Vec2::new(16 + 3, 16), 2);

        // Get the entities back as pairs by flushing all buckets
        let pairs = grid.flush().collect::<Vec<_>>();
        assert!(pairs.contains(&(0, 1)));
        assert!(pairs.contains(&(0, 2)));
        assert!(pairs.contains(&(1, 2)));

        // When we store 4 entities the last one should be dropped because of the max bucket size
        for _ in 0..3 {
            grid.store_entity(Vec2::new(0, 0), 0);
        }
        grid.store_entity(Vec2::new(0, 0), 2);

        // Get the entities back as pairs by flushing all buckets
        let pairs = grid.flush().collect::<Vec<_>>();
        assert!(!pairs.contains(&(0, 2)));
    }

    /// Test different shapes.
    #[test]
    fn test_shapes() {
        const WIDTH: u16 = 100;
        const HEIGHT: u16 = 100;
        const STEP: u16 = 10;
        const BUCKET: usize = 3;
        const SIZE: usize = (WIDTH / STEP * HEIGHT / STEP) as usize;

        let mut grid = SpatialGrid::<u8, WIDTH, HEIGHT, STEP, BUCKET, SIZE>::new();

        // Spawn multiple overlapping rectangles
        grid.store_aabr(Vec2::new(10, 10), Extent2::new(80, 20), 0);
        grid.store_aabr(Vec2::new(10, 29), Extent2::new(20, 70), 1);

        // Get the entities back as pairs by flushing all buckets
        let pairs = grid.flush().collect::<Vec<_>>();
        assert!(pairs.contains(&(0, 1)));
    }
}

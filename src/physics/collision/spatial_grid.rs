use std::{
    collections::HashSet,
    fmt::{Debug, Display},
    hash::Hash,
};

use arrayvec::ArrayVec;
use itertools::Itertools;
use vek::{Aabr, Vec2};

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
    /// Amount of horizontal slots.
    pub const STEPPED_WIDTH: u16 = WIDTH / STEP;
    /// Amount of vertical slots.
    pub const STEPPED_HEIGHT: u16 = HEIGHT / STEP;

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

    /// Drop everything from the buckets.
    pub fn clear(&mut self) {
        for bucket in self.buckets.iter_mut() {
            // Remove everything from the bucket
            bucket.clear();
        }
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

    /// Flush all buckets into a vector of all matching pairs.
    ///
    /// The list of matching pairs doesn't contain the same pairs twice.
    pub fn flush_into(&mut self, pairs: &mut Vec<(I, I)>) {
        // Keep track of the already matching collision pairs
        let mut added = HashSet::new();

        for bucket in self.buckets.iter_mut() {
            // Combine all items in the bucket
            bucket
                // Remove everything from the bucket
                .take()
                .into_iter()
                // Get all possible combinations of values in the bucket as tuples
                .tuple_combinations()
                // We don't have to check the order of the pair because the order of entry is guaranteed to be the same for earlier intersections
                .for_each(|pair: (I, I)| {
                    if !added.contains(&pair) {
                        added.insert(pair);

                        pairs.push(pair);
                    }
                });
        }
    }

    /// Store an entity AABR rectangle.
    ///
    /// This will fill all buckets that are colliding with this rectangle.
    ///
    /// Drops an entity when the bucket is full or when it's outside of the range.
    pub fn store_aabr(&mut self, aabr: Aabr<i16>, id: I) {
        puffin::profile_function!();

        // Ignore anything fully outside of the grid
        if !self.is_aabr_in_range(aabr) {
            return;
        }

        // Clamp the rectangle within the grid
        let edge = Vec2::new(
            Self::STEPPED_WIDTH as i16 - 1,
            Self::STEPPED_HEIGHT as i16 - 1,
        );
        let start: Vec2<i16> = Vec2::min(Vec2::max(aabr.min / STEP as i16, Vec2::zero()), edge);
        let end: Vec2<i16> = Vec2::min(Vec2::max(aabr.max / STEP as i16, Vec2::zero()), edge);

        for y in start.y..=end.y {
            for x in start.x..=end.x {
                self.add_to_bucket(x as u16 + y as u16 * Self::STEPPED_WIDTH, id);
            }
        }
    }

    /// Whether an AABR can be stored.
    pub fn is_aabr_in_range(&self, aabr: Aabr<i16>) -> bool {
        puffin::profile_function!();

        // Ignore anything fully outside of the grid
        !(aabr.max.x < 0
            || aabr.max.y < 0
            || aabr.min.x >= WIDTH as i16
            || aabr.min.y >= HEIGHT as i16)
    }

    /// Get a debug map 2D grid where each value is the amount of items in the bucket.
    ///
    /// Dimensions are [`Self::STEPPED_WIDTH`] * [`Self::STEPPED_HEIGHT`].
    pub fn amount_map(&self) -> Vec<u8> {
        self.buckets
            .iter()
            .map(|bucket| bucket.len() as u8)
            .collect()
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

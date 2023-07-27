use arrayvec::ArrayVec;
use vek::{Extent2, Vec2};

/// Spatial hash grid with fixed buckets divided over an area so potential collision pairs can be found quickly.
///
/// Entities gets dropped when added to buckets that are already full.
///
/// Because of not allowing arithmetic (yet) in Rust const generics the following needs to be calculated:
/// - `SIZE` is `(WIDTH * HEIGHT) / STEP`.
/// - `STEP` is how many pixels fit in each bucket, must be a power of 2
/// - `BUCKET` is amount of simultaneous objects can be checked at the same time
/// - `I` is the type for identifying another object. It's smart to keep this as small as possible
pub struct SpatialGrid<
    I,
    const WIDTH: usize,
    const SIZE: usize,
    const STEP: usize,
    const BUCKET: usize,
> where
    I: Copy,
{
    /// Buckets spread out over the grid.
    buckets: [ArrayVec<I, BUCKET>; SIZE],
}

impl<I, const WIDTH: usize, const SIZE: usize, const STEP: usize, const BUCKET: usize>
    SpatialGrid<I, WIDTH, SIZE, STEP, BUCKET>
where
    I: Copy,
{
    const HEIGHT: usize = { (SIZE / WIDTH) * STEP };

    /// Construct a new grid.
    // TODO: wait for either `.map` or `std::array::from_fn` to become const generic to make this const generic.
    pub fn new() -> Self {
        assert_eq!((WIDTH * Self::HEIGHT) / STEP, SIZE);
        assert!(STEP.is_power_of_two());
        assert!(STEP >= 4);

        let buckets = std::array::from_fn(|_| ArrayVec::new_const());

        Self { buckets }
    }

    /// Store an entity AABB rectangle.
    ///
    /// Drops an entity when the bucket is full.
    ///
    /// Panics when the object is outside of the size of the grid.
    pub fn store_aabb(&mut self, pos: Vec2<u16>, size: Extent2<u16>, id: I) {}

    /// Store a single entity.
    ///
    /// Drops an entity when the bucket is full.
    ///
    /// Panics when the entity is outside of the size of the grid.
    pub fn store_entity(&mut self, pos: Vec2<u16>, id: I) {
        let index = Self::coordinate_to_index(pos);

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
    fn coordinate_to_index(coord: Vec2<u16>) -> u32 {
        /// Dividy by step size
        let x = coord.x / (STEP as u16);
        let y = coord.y / (STEP as u16);
        debug_assert!(x < (WIDTH / STEP) as u16);
        debug_assert!(y < (Self::HEIGHT / STEP) as u16);

        x as u32 * y as u32
    }
}

/// Test different grid constructions.
#[test]
fn test_insertion() {
    const WIDTH: usize = 100;
    const HEIGHT: usize = 100;
    const STEP: usize = 16;

    let mut grid = SpatialGrid::<u8, WIDTH, { WIDTH * HEIGHT }, STEP, 1>::new();

    // Store 2 entities in the same bucket
    grid.store_entity(Vec2::new(30, 30), 0);
    grid.store_entity(Vec2::new(31, 30), 1);
}

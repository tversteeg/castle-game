use bevy::utils::tracing;
use geo::{prelude::BoundingRect, LineString};
use geo_booleanop::boolean::BooleanOp;
use geo_types::{Coordinate, Polygon, Rect};
use rand::Rng;

/// Split a polygon into multiple parts.
pub trait Split<T> {
    // TODO: https://github.com/rust-lang/rust/issues/63063
    // type Iter: Iterator<Item = T>;

    /// Split the polygon into multiple parts by creating a random shape and using boolean
    /// operations.
    fn split(&self) -> Box<dyn Iterator<Item = T>>;
}

impl Split<Polygon<f32>> for Polygon<f32> {
    // TODO: https://github.com/rust-lang/rust/issues/63063
    // type Iter = impl Iterator<Item = Self>;

    #[tracing::instrument(name = "splitting polygon", level = "info")]
    fn split(&self) -> Box<dyn Iterator<Item = Self>> {
        // Setup the random generator
        let mut rng = rand::thread_rng();

        // Create a random polygon shape through the center
        let bounding_rect = self
            .bounding_rect()
            // Use a small rectangle when the bounding rectangle can't be calculated, this shouldn't
            // happen much
            .unwrap_or_else(|| {
                Rect::new(Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 1.0, y: 1.0 })
            });

        // Add a new point to the at a random point
        let min = bounding_rect.min();
        let max = bounding_rect.max();

        // Create the random polygon
        let random_shape = Polygon::new(
            LineString::from(vec![
                (
                    rng.gen_range::<f32, _>(min.x..max.x),
                    rng.gen_range::<f32, _>(min.y..max.y),
                ),
                (max.x, min.y),
                (max.x, max.y),
                (min.x, max.y),
            ]),
            vec![],
        );

        // Get both sides of the object through boolean operations
        let side1 = self.intersection(&random_shape);
        let side2 = self.difference(&random_shape);

        // Get all polygons from both splits
        Box::new(side1.into_iter().chain(side2.into_iter()))
    }
}

use bevy::utils::tracing;
use geo::prelude::BoundingRect;
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

        // Convert the bounding box to a polygon so we can play with the coordinates
        let mut random_shape = bounding_rect.to_polygon();

        // Add a new point to the at a random point
        let min = bounding_rect.min();
        let max = bounding_rect.max();
        random_shape.exterior_mut(|exterior| {
            exterior.0[1] = Coordinate {
                x: rng.gen_range::<f32, _>(min.x..max.x),
                y: rng.gen_range::<f32, _>(min.y..max.y),
            };
        });

        // Get both sides of the object through boolean operations
        let side1 = self.intersection(&random_shape);
        let side2 = self.difference(&random_shape);

        // Get all polygons from both splits
        Box::new(side1.into_iter().chain(side2.into_iter()))
    }
}

//! Separating axis theorem.

use vek::Vec2;

use super::shape::Rectangle;

/// Make a shape implement collision detection with other shapes.
///
/// Implemented with Separating Axis Theorem.
pub trait NarrowCollision {
    /// Check for collision with a rectangle, returning collision information.
    ///
    /// Returns `None` when no collision.
    fn collide_rectangle(
        &self,
        pos: Vec2<f32>,
        rot: f32,
        other_rect: Rectangle,
        other_pos: Vec2<f32>,
        other_rot: f32,
    ) -> Option<CollisionResponse>;
}

/// Response for a collision.
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionResponse {}

/// A simple projection on an axis that can be used to check for overlaps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    /// Minimum projected value.
    min: f32,
    /// Maximum projected value.
    max: f32,
}

impl Projection {
    /// Project a polygon shape on an axis.
    pub fn project<const SIZE: usize>(vertices: [Vec2<f32>; SIZE], axis: Vec2<f32>) -> Self {
        // Start by projecting the first so we don't have to check for infinite
        let mut min = axis.dot(vertices[0]);
        let mut max = min;

        // Skip the first because we already token that for min
        for vertex in vertices.into_iter().skip(1) {
            let proj = axis.dot(vertex);

            min = min.min(proj);
            max = max.max(proj);
        }

        Self { min, max }
    }

    /// Check if this overlaps another axis.
    pub fn separated(&self, other: Self) -> bool {
        other.max < self.min || self.max < other.min
    }
}

//! Separating axis theorem.

use vek::Vec2;

use crate::math::Rotation;

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
        rot: Rotation,
        other_rect: Rectangle,
        other_pos: Vec2<f32>,
        other_rot: Rotation,
    ) -> Option<CollisionResponse>;
}

/// Response for a collision.
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionResponse {
    /// Vertex from which the vector can be cast.
    pub contact: Vec2<f32>,
    /// Minimum translation vector.
    pub mtv: Vec2<f32>,
}

/// A simple projection on an axis that can be used to check for overlaps.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    /// Minimum projected value.
    pub min: f32,
    /// Maximum projected value.
    pub max: f32,
    /// Minimum vertex that's projected.
    pub min_vertex: Vec2<f32>,
    /// Maximum vertex that's projected.
    pub max_vertex: Vec2<f32>,
}

impl Projection {
    /// Project a polygon shape on an axis.
    pub fn project<const SIZE: usize>(vertices: [Vec2<f32>; SIZE], axis: Vec2<f32>) -> Self {
        puffin::profile_function!();

        // Start by projecting the first so we don't have to check for infinite
        let mut min = axis.dot(vertices[0]);
        let mut max = min;
        let mut min_vertex = vertices[0];
        let mut max_vertex = min_vertex;

        // Skip the first because we already token that for min
        for vertex in vertices.into_iter().skip(1) {
            let proj = axis.dot(vertex);

            if proj < min {
                min = proj;
                min_vertex = vertex;
            }
            if proj > max {
                max = proj;
                max_vertex = vertex;
            }
        }

        Self {
            min,
            max,
            min_vertex,
            max_vertex,
        }
    }

    /// Check if this overlaps another axis.
    pub fn separated(&self, other: Self) -> bool {
        other.max < self.min || self.max < other.min
    }

    /// Calculate the overlap.
    pub fn overlap(&self, other: Self) -> f32 {
        (self.max.min(other.max) - self.min.max(other.min)).max(0.0)
    }
}

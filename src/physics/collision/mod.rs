pub mod shape;
pub mod spatial_grid;

use arrayvec::ArrayVec;
use vek::Vec2;

use crate::math::Rotation;

use self::shape::Rectangle;

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
    ) -> ArrayVec<CollisionResponse, 2>;
}

/// Response for a collision.
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionResponse {
    /// Local position of contact point 1.
    pub local_contact_1: Vec2<f32>,
    /// Local position of contact point 2.
    pub local_contact_2: Vec2<f32>,
    /// Normalized direction of collision.
    pub normal: Vec2<f32>,
    /// Distance of penetration between objects.
    pub penetration: f32,
}

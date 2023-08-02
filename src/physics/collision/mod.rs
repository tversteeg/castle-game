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
    /// Minimum translation vector.
    pub mtv: Vec2<f32>,
}

impl CollisionResponse {
    /// Transform to world position on body 1.
    fn local_contact_1_to_world(&self, pos: Vec2<f32>, rot: Rotation) -> Vec2<f32> {
        pos + rot.rotate(self.local_contact_1)
    }

    /// Transform to world position on body 2.
    fn local_contact_2_to_world(&self, pos: Vec2<f32>, rot: Rotation) -> Vec2<f32> {
        pos + rot.rotate(self.local_contact_2)
    }
}

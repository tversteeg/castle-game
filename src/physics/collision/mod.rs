pub mod shape;
pub mod spatial_grid;

use vek::Vec2;

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

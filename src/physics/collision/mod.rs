pub mod shape;
pub mod spatial_grid;

use parry2d_f64::query::{ContactManifold, ContactManifoldsWorkspace};
use vek::Vec2;

/// Collision state that persists over multiple detections.
///
/// Used to improve parry performance.
pub struct CollisionState<K> {
    /// Calculated manifolds cache.
    pub manifolds: Vec<ContactManifold<(), ()>>,
    /// Detected collisions.
    pub collisions: Vec<(K, K, CollisionResponse)>,
}

impl<K> CollisionState<K> {
    /// Construct a new cache.
    pub fn new() -> Self {
        let manifolds = Vec::with_capacity(16);
        let collisions = Vec::new();

        Self {
            manifolds,
            collisions,
        }
    }

    /// Clear all detected collisions.
    pub fn clear(&mut self) {
        self.collisions.clear();
    }
}

/// Response for a collision.
#[derive(Debug, Clone, PartialEq)]
pub struct CollisionResponse {
    /// Local position of contact point 1.
    pub local_contact_1: Vec2<f64>,
    /// Local position of contact point 2.
    pub local_contact_2: Vec2<f64>,
    /// Normalized direction of collision.
    pub normal: Vec2<f64>,
    /// Distance of penetration between objects.
    pub penetration: f64,
}

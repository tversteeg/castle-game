pub mod shape;

use std::{collections::HashSet, hash::Hash};

use parry2d_f64::query::ContactManifold;
use vek::Vec2;

use crate::math::Iso;

use self::shape::Shape;

/// Collision state that persists over multiple detections.
///
/// Used to improve parry performance.
pub struct CollisionState<K> {
    /// Calculated manifolds cache.
    pub manifolds: Vec<ContactManifold<(), ()>>,
    /// Detected collisions in a substep.
    pub substep_collisions: Vec<(K, K, CollisionResponse)>,
    /// Detected collisions in a single step.
    pub step_collisions: HashSet<(K, K)>,
}

impl<K> CollisionState<K> {
    /// Construct a new cache.
    pub fn new() -> Self {
        let manifolds = Vec::with_capacity(16);
        let substep_collisions = Vec::new();
        let step_collisions = HashSet::new();

        Self {
            manifolds,
            substep_collisions,
            step_collisions,
        }
    }

    /// Clear all detected collisions in a substep.
    pub fn clear_substep(&mut self) {
        self.substep_collisions.clear();
    }

    /// Clear all detected collisions in a full step.
    pub fn clear_step(&mut self) {
        self.step_collisions.clear();
    }

    /// Detect a new collision based on a broad-phase detected pair.
    pub fn detect(
        &mut self,
        a_data: K,
        a_shape: &Shape,
        a_pos: Iso,
        b_data: K,
        b_shape: &Shape,
        b_pos: Iso,
    ) where
        K: Clone + Hash + Eq,
    {
        a_shape.push_collisions(a_pos, a_data, b_shape, b_pos, b_data, self);
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

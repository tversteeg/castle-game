use slotmap::HopSlotMap;
use vek::Vec2;

use crate::physics::{rigidbody::RigidBody, RigidBodyKey};

use super::{Constraint, PositionalConstraint};

/// Constraint that always tries to keep rigidbodies at a certain distance from each other.
#[derive(Debug, Clone)]
pub struct DistanceConstraint {
    /// Object A.
    a: RigidBodyKey,
    /// Object B.
    b: RigidBodyKey,
    /// Attachment point A.
    a_attachment: Vec2<f64>,
    /// Attachment point B.
    b_attachment: Vec2<f64>,
    /// Distance the constraint tries to resolve to.
    rest_dist: f64,
    /// Factor of how fast the distance is resolved.
    ///
    /// Inverse of stiffness.
    compliance: f64,
    /// Lambda value.
    ///
    /// Must be reset every frame.
    lambda: f64,
}

impl DistanceConstraint {
    /// Constrain two rigidbodies with a spring so they can't be try to resolve the distance between them.
    ///
    /// Attachment point is offset from the center at rotation zero where the constraint will be attached to.
    ///
    /// RigidBodys must be indices.
    pub fn new(
        a: RigidBodyKey,
        a_attachment: Vec2<f64>,
        b: RigidBodyKey,
        b_attachment: Vec2<f64>,
        rest_dist: f64,
        compliance: f64,
    ) -> Self {
        let lambda = 0.0;

        Self {
            a,
            b,
            a_attachment,
            b_attachment,
            lambda,
            rest_dist,
            compliance,
        }
    }

    /// Get the attachments in world-space.
    pub fn attachments_world(
        &self,
        rigidbodies: &HopSlotMap<RigidBodyKey, RigidBody>,
    ) -> (Vec2<f64>, Vec2<f64>) {
        let [a, b] = self.rigidbodies(rigidbodies);

        (
            a.local_to_world(self.a_attachment),
            b.local_to_world(self.b_attachment),
        )
    }
}

impl Constraint<2> for DistanceConstraint {
    fn solve(&mut self, rigidbodies: &mut HopSlotMap<RigidBodyKey, RigidBody>, dt: f64) {
        puffin::profile_function!("Solve distance constraint");

        let [a, b] = self.rigidbodies_mut(rigidbodies);

        // Ignore sleeping or static bodies
        if !a.is_active() && !b.is_active() {
            return;
        }

        self.apply(a, self.a_attachment, b, self.b_attachment, dt);
    }

    fn lambda(&self) -> f64 {
        self.lambda
    }

    fn set_lambda(&mut self, lambda: f64) {
        self.lambda = lambda;
    }

    fn rigidbody_keys(&self) -> [RigidBodyKey; 2] {
        [self.a, self.b]
    }
}

impl PositionalConstraint for DistanceConstraint {
    fn gradient(&self, a_world_position: Vec2<f64>, b_world_position: Vec2<f64>) -> Vec2<f64> {
        // Vector pointing away from the other rigidbody
        let delta = a_world_position - b_world_position;

        // Normalize or point at some position
        delta.try_normalized().unwrap_or(Vec2::unit_y())
    }

    fn magnitude(&self, a_world_position: Vec2<f64>, b_world_position: Vec2<f64>) -> f64 {
        // Difference between rest distance and actual distance
        let dist = a_world_position.distance(b_world_position);

        dist - self.rest_dist
    }

    fn compliance(&self) -> f64 {
        self.compliance
    }
}

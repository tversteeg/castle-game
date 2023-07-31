use hashbrown::HashMap;
use vek::Vec2;

use crate::physics::rigidbody::{RigidBody, RigidBodyIndex};

use super::{Constraint, PositionalConstraint};

/// Constraint that always tries to keep rigidbodies at a certain distance from each other.
#[derive(Debug, Clone)]
pub struct DistanceConstraint {
    /// Object A.
    a: RigidBodyIndex,
    /// Object B.
    b: RigidBodyIndex,
    /// Attachment point A.
    a_attachment: Vec2<f32>,
    /// Attachment point B.
    b_attachment: Vec2<f32>,
    /// Distance the constraint tries to resolve to.
    rest_dist: f32,
    /// Factor of how fast the distance is resolved.
    ///
    /// Inverse of stiffness.
    compliance: f32,
    /// Lambda value.
    ///
    /// Must be reset every frame.
    lambda: f32,
}

impl DistanceConstraint {
    /// Constrain two rigidbodies with a spring so they can't be try to resolve the distance between them.
    ///
    /// Attachment point is offset from the center at rotation zero where the constraint will be attached to.
    ///
    /// RigidBodys must be indices.
    pub fn new(
        a: RigidBodyIndex,
        a_attachment: Vec2<f32>,
        b: RigidBodyIndex,
        b_attachment: Vec2<f32>,
        rest_dist: f32,
        compliance: f32,
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
        rigidbodies: &HashMap<RigidBodyIndex, RigidBody>,
    ) -> (Vec2<f32>, Vec2<f32>) {
        let (a, b) = self.rigidbodies(rigidbodies);

        (
            a.local_to_world(self.a_attachment),
            b.local_to_world(self.b_attachment),
        )
    }

    /// Get the rigidbodies as a tuple from the hashmap.
    fn rigidbodies<'a>(
        &self,
        rigidbodies: &'a HashMap<RigidBodyIndex, RigidBody>,
    ) -> (&'a RigidBody, &'a RigidBody) {
        let a = rigidbodies
            .get(&self.a)
            .expect("Couldn't get rigidbody 'a' for distance constraint");
        let b = rigidbodies
            .get(&self.b)
            .expect("Couldn't get rigidbody 'b' for distance constraint");

        (a, b)
    }
}

impl Constraint for DistanceConstraint {
    fn solve(&mut self, rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>, dt: f32) {
        let [a, b] = rigidbodies
            .get_many_mut([&self.a, &self.b])
            .expect("Couldn't get rigidbodies for distance constraint");

        self.apply(a, self.a_attachment, b, self.b_attachment, dt);
    }

    fn lambda(&self) -> f32 {
        self.lambda
    }

    fn set_lambda(&mut self, lambda: f32) {
        self.lambda = lambda;
    }
}

impl PositionalConstraint for DistanceConstraint {
    fn gradient(&self, a_world_position: Vec2<f32>, b_world_position: Vec2<f32>) -> Vec2<f32> {
        // Vector pointing away from the other rigidbody
        let delta = a_world_position - b_world_position;

        // Normalize or point at some position
        delta.try_normalized().unwrap_or(Vec2::unit_y())
    }

    fn magnitude(&self, a_world_position: Vec2<f32>, b_world_position: Vec2<f32>) -> f32 {
        // Difference between rest distance and actual distance
        let dist = a_world_position.distance(b_world_position);

        dist - self.rest_dist
    }

    fn compliance(&self) -> f32 {
        self.compliance
    }
}

use std::collections::HashMap;

use vek::Vec2;

use crate::physics::{
    collision::sat::CollisionResponse,
    rigidbody::{RigidBody, RigidBodyIndex},
};

use super::{Constraint, PositionalConstraint};

/// Short-lived collision constraint for resolving collisions.
#[derive(Debug, Clone)]
pub struct PenetrationConstraint {
    /// Object A.
    a: RigidBodyIndex,
    /// Object B.
    b: RigidBodyIndex,
    /// Collision response.
    response: CollisionResponse,
    /// Lambda value.
    ///
    /// Must be reset every frame.
    lambda: f32,
}

impl PenetrationConstraint {
    /// Constrain two rigidbodies with a spring so they can't be try to resolve the distance between them.
    ///
    /// RigidBodys must be indices.
    pub fn new(rigidbodies: [RigidBodyIndex; 2], response: CollisionResponse) -> Self {
        let lambda = 0.0;
        let [a, b] = rigidbodies;

        Self {
            lambda,
            a,
            b,
            response,
        }
    }
}

impl Constraint for PenetrationConstraint {
    fn solve(&mut self, rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>, dt: f32) {
        let [mut a, mut b] = rigidbodies
            .get_many_mut([&self.a, &self.b])
            .expect("Couldn't get rigidbodies for penetration constraint");

        self.apply(
            &mut a,
            self.response.contact,
            &mut b,
            self.response.contact,
            dt,
        );
    }
    fn lambda(&self) -> f32 {
        self.lambda
    }

    fn set_lambda(&mut self, lambda: f32) {
        self.lambda = lambda;
    }
}

impl PositionalConstraint for PenetrationConstraint {
    fn gradient(&self, _a_global_position: Vec2<f32>, _b_global_position: Vec2<f32>) -> Vec2<f32> {
        // MTV vector from the collision normalized
        -self.response.mtv.normalized()
    }

    fn magnitude(&self, _a_global_position: Vec2<f32>, _b_global_position: Vec2<f32>) -> f32 {
        // MTV vector length which is how far we need to travel to resolve the collision
        self.response.mtv.magnitude()
    }

    fn compliance(&self) -> f32 {
        0.001
    }
}

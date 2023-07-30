use std::collections::HashMap;

use vek::Vec2;

use crate::physics::rigidbody::{RigidBody, RigidBodyIndex};

use super::Constraint;

/// Constraint that stops the rigid bodies from touching the ground.
#[derive(Debug, Clone)]
pub struct GroundConstraint {
    /// Y value of the ground.
    height: f32,
    /// Index of the rigidbody.
    rigidbody: [RigidBodyIndex; 1],
    /// Lambda value.
    ///
    /// Must be reset every frame.
    lambda: f32,
}

impl GroundConstraint {
    /// Stop the rigid body from falling through the ground.
    pub fn new(rigidbody: RigidBodyIndex, height: f32) -> Self {
        let lambda = 0.0;
        let rigidbody = [rigidbody];

        Self {
            lambda,
            rigidbody,
            height,
        }
    }

    fn gradients(&self, _rigidbodies_pos: [Vec2<f32>; 1]) -> [Vec2<f32>; 1] {
        // Always point down
        [Vec2::unit_y()]
    }

    fn constraint(&self, rigidbodies_pos: [Vec2<f32>; 1]) -> f32 {
        if rigidbodies_pos[0].y < self.height {
            // Not touching the ground, don't apply force
            0.0
        } else {
            rigidbodies_pos[0].y - self.height
        }
    }

    fn attachment_points(&self, rigidbodies_pos: [Vec2<f32>; 1]) -> [Vec2<f32>; 1] {
        [Vec2::new(rigidbodies_pos[0].x, self.height)]
    }

    fn rigidbodies(&self) -> &[RigidBodyIndex; 1] {
        &self.rigidbody
    }

    fn compliance(&self) -> f32 {
        // The ground is not very flexible
        0.0
    }
}

impl Constraint for GroundConstraint {
    fn solve(&mut self, rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>, dt: f32) {
        let rigidbodies_pos = self
            .rigidbodies()
            .map(|rigidbody_index| rigidbodies[&rigidbody_index].position());

        let rigidbodies_inv_mass = self
            .rigidbodies()
            .map(|rigidbody_index| rigidbodies[&rigidbody_index].inverse_mass());

        // All massess combined
        let sum_inv_mass: f32 = rigidbodies_inv_mass.iter().sum();
        if sum_inv_mass == 0.0 {
            // Nothing to do since there's no mass
            return;
        }

        let stiffness = self.compliance() / dt.powi(2);

        let gradients = self.gradients(rigidbodies_pos);

        // Sum of all inverse masses multiplied by the squared lengths of the matching gradients
        let w_sum = rigidbodies_inv_mass
            .iter()
            .zip(gradients)
            .map(|(inv_mass, gradient)| inv_mass * gradient.magnitude_squared())
            .sum::<f32>();

        if w_sum == 0.0 {
            // Avoid divisions by zero
            return;
        }

        // Previous lambda value
        let lambda = self.lambda();

        // XPBD Lagrange lambda, signed magnitude of the correction
        let delta_lambda =
            (-self.constraint(rigidbodies_pos) - stiffness * lambda) / (w_sum + stiffness);

        // Store the result for other sub-steps
        self.set_lambda(lambda + delta_lambda);

        // How much the rigidbody should move to try to satisfy the constraint
        let correction_vectors = gradients.map(|gradient| gradient * delta_lambda);

        // Apply offsets to rigidbodies
        correction_vectors
            .iter()
            .zip(self.rigidbodies())
            .zip(self.attachment_points(rigidbodies_pos))
            .for_each(|((correction_vector, rigidbody_index), attachment_point)| {
                let rigidbody = rigidbodies
                    .get_mut(rigidbody_index)
                    .expect("RigidBody index mismatch");

                // Perpendicular position
                let perp_dot_pos = (attachment_point.x * correction_vector.y)
                    - (attachment_point.y * correction_vector.x);

                let inv_inertia = rigidbody.inverse_inertia();
                let generalized_inverse_mass =
                    rigidbody.inverse_mass() + inv_inertia * perp_dot_pos.powi(2);

                // Apply the solved forces on the rigidbody
                rigidbody.apply_force(correction_vector * generalized_inverse_mass);
                rigidbody.apply_rotational_force(inv_inertia * perp_dot_pos);
            });
    }

    fn lambda(&self) -> f32 {
        self.lambda
    }

    fn set_lambda(&mut self, lambda: f32) {
        self.lambda = lambda;
    }
}

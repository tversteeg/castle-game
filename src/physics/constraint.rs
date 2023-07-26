use vek::Vec2;

use super::{RigidBody, RigidBodyIndex};

/// XPBD constraint between one or more rigidbodies.
pub trait Constraint<const RIGIDBODY_COUNT: usize> {
    /// RigidBody indices this constraint applies to.
    fn rigidbodies(&self) -> &[RigidBodyIndex; RIGIDBODY_COUNT];

    /// Normalized vectors pointing to the least-optimal solution for solving the constraint.
    fn gradients(
        &self,
        rigidbodies_pos: [Vec2<f64>; RIGIDBODY_COUNT],
    ) -> [Vec2<f64>; RIGIDBODY_COUNT];

    /// Error value, when the value is zero it's resolved and the constraint isn't active.
    fn constraint(&self, rigidbodies_pos: [Vec2<f64>; RIGIDBODY_COUNT]) -> f64;

    /// Factor of how fast the distance is resolved.
    ///
    /// Inverse of stiffness.
    fn compliance(&self) -> f64;

    /// Current stored lambda.
    fn lambda(&self) -> f64;

    /// Set the lambda.
    fn set_lambda(&mut self, lambda: f64);

    /// Solve the constraint.
    ///
    /// Applies the force immediately to the rigidbodies.
    ///
    //// Returns the global lambda with the added local lambda.
    // TODO: make the Vec stack-allocated by referencing the rigidbodies directly
    // TODO: reduce amount of zip operations
    fn solve(&mut self, rigidbodies: &mut [RigidBody], dt: f64) {
        let rigidbodies_pos = self
            .rigidbodies()
            .map(|rigidbody_index| rigidbodies[rigidbody_index as usize].position());

        let rigidbodies_inv_mass = self
            .rigidbodies()
            .map(|rigidbody_index| rigidbodies[rigidbody_index as usize].inverse_mass());

        // All massess combined
        let sum_inv_mass: f64 = rigidbodies_inv_mass.iter().sum();
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
            .sum::<f64>();

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
        correction_vectors.iter().zip(self.rigidbodies()).for_each(
            |(correction_vector, rigidbody_index)| {
                let rigidbody = rigidbodies
                    .get_mut(*rigidbody_index as usize)
                    .expect("RigidBody index mismatch");

                // Apply the rotation
                let inv_inertia = rigidbody.inertia().recip();
                let pos = rigidbody.position();
                // This needs to be a point of impact
                // Perpendicular position
                let perp_dot_pos = (pos.x * pos.y) - (pos.y * pos.x);

                let generalized_inverse_mass =
                    rigidbody.inverse_mass() + inv_inertia * perp_dot_pos.powi(2);

                // Apply the solved forces on the rigidbody
                rigidbody.apply_force(correction_vector * generalized_inverse_mass);
                rigidbody.apply_rotational_force(inv_inertia * perp_dot_pos);
            },
        );
    }

    /// Reset the constraint at the beginning of a step (not a sub-step).
    fn reset(&mut self) {
        self.set_lambda(0.0);
    }
}

/// Constraint that always tries to keep rigidbodies at a certain distance from each other.
#[derive(Debug, Clone)]
pub struct DistanceConstraint {
    /// Indices of the rigidbodies.
    rigidbodies: [RigidBodyIndex; 2],
    /// Distance the contstraint tries to resolve to.
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
    /// RigidBodys must be indices.
    pub fn new(rigidbodies: [RigidBodyIndex; 2], rest_dist: f64, compliance: f64) -> Self {
        let lambda = 0.0;

        Self {
            lambda,
            rigidbodies,
            rest_dist,
            compliance,
        }
    }
}

impl Constraint<2> for DistanceConstraint {
    fn gradients(&self, rigidbodies_pos: [Vec2<f64>; 2]) -> [Vec2<f64>; 2] {
        // Vector pointing away from the other rigidbody
        let delta = rigidbodies_pos[0] - rigidbodies_pos[1];
        // Normalize or zero
        let dir = delta.try_normalized().unwrap_or_default();

        [dir, -dir]
    }

    fn constraint(&self, rigidbodies_pos: [Vec2<f64>; 2]) -> f64 {
        // Difference between rest distance and actual distance
        let dist = rigidbodies_pos[0].distance(rigidbodies_pos[1]);

        dist - self.rest_dist
    }

    fn rigidbodies(&self) -> &[RigidBodyIndex; 2] {
        &self.rigidbodies
    }

    fn compliance(&self) -> f64 {
        self.compliance
    }

    fn lambda(&self) -> f64 {
        self.lambda
    }

    fn set_lambda(&mut self, lambda: f64) {
        self.lambda = lambda;
    }
}

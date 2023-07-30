pub mod distance;
pub mod penetration;

use std::collections::HashMap;

use vek::Vec2;

use super::{collision::sat::CollisionResponse, RigidBody, RigidBodyIndex};

/// Constraint index type.
pub type ConstraintIndex = u32;

/// XPBD constraint between one or more rigid bodies.
pub trait Constraint {
    /// Current stored lambda.
    fn lambda(&self) -> f32;

    /// Set the lambda.
    fn set_lambda(&mut self, lambda: f32);

    /// Solve the constraint.
    ///
    /// Applies the force immediately to the rigidbodies.
    fn solve(&mut self, rigidbodies: &mut HashMap<RigidBodyIndex, RigidBody>, dt: f32);

    /// Reset the constraint at the beginning of a step (not a sub-step).
    fn reset(&mut self) {
        self.set_lambda(0.0);
    }
}

/// Constraint specialization for position restrictions.
pub trait PositionalConstraint: Constraint {
    /// Direction normalized vector.
    fn gradient(&self, a_world_position: Vec2<f32>, b_world_position: Vec2<f32>) -> Vec2<f32>;

    /// Magnitude.
    fn magnitude(&self, a_world_position: Vec2<f32>, b_world_position: Vec2<f32>) -> f32;

    /// Compliance.
    fn compliance(&self) -> f32;

    /// Calculate and apply the forces from the implemented methods.
    ///
    /// Updates the lambda.
    fn apply(
        &mut self,
        a: &mut RigidBody,
        a_attachment: Vec2<f32>,
        b: &mut RigidBody,
        b_attachment: Vec2<f32>,
        dt: f32,
    ) {
        let a_world_position = a.local_to_world(a_attachment);
        let b_world_position = b.local_to_world(b_attachment);

        let gradient = self.gradient(a_world_position, b_world_position);

        // Rotate the attachments
        let a_attachment = a.rotate(a_attachment);
        let b_attachment = b.rotate(b_attachment);

        let a_generalized_inverse_mass = a.inverse_mass_at_relative_point(a_attachment, gradient);
        let b_generalized_inverse_mass = b.inverse_mass_at_relative_point(b_attachment, gradient);

        let stiffness = self.compliance() / dt.powi(2);

        let delta_lambda = (-self.magnitude(a_world_position, b_world_position)
            - stiffness * self.lambda())
            / (a_generalized_inverse_mass + b_generalized_inverse_mass + stiffness);

        if delta_lambda == 0.0 {
            // Nothing will change, do nothing
            return;
        }

        // lambda += delta_lambda
        self.set_lambda(self.lambda() + delta_lambda);

        let positional_impulse = gradient * delta_lambda;

        // Change positions of bodies
        a.apply_force(positional_impulse * a.inverse_mass());
        b.apply_force(-positional_impulse * b.inverse_mass());

        // Change rotation of bodies
        a.apply_rotational_force(a.delta_rotation_at_point(a_attachment, positional_impulse));
        b.apply_rotational_force(-b.delta_rotation_at_point(b_attachment, positional_impulse));
    }
}

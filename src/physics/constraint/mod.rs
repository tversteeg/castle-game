pub mod distance;
pub mod penetration;

use hashbrown::HashMap;
use vek::Vec2;

use super::{RigidBody, RigidBodyIndex};

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

    /// Calculate a generic form of the lambda update.
    fn delta_lambda<const AMOUNT: usize>(
        lambda: f32,
        magnitude: f32,
        compliance: f32,
        gradients: [Vec2<f32>; AMOUNT],
        attachments: [Vec2<f32>; AMOUNT],
        bodies: [&RigidBody; AMOUNT],
        dt: f32,
    ) -> f32 {
        puffin::profile_function!();

        let generalized_inverse_mass_sum: f32 = gradients
            .into_iter()
            .zip(attachments)
            .zip(bodies)
            .map(|((gradient, attachment), body)| {
                body.inverse_mass_at_relative_point(attachment, gradient)
            })
            .sum();

        if generalized_inverse_mass_sum <= std::f32::EPSILON {
            // Avoid divisions by zero
            return 0.0;
        }

        let stiffness = compliance / dt.powi(2);

        (-magnitude - stiffness * lambda) / (generalized_inverse_mass_sum + stiffness)
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
        puffin::profile_function!();

        let a_world_position = a.local_to_world(a_attachment);
        let b_world_position = b.local_to_world(b_attachment);

        let gradient = self.gradient(a_world_position, b_world_position);

        // Rotate the attachments
        let a_attachment = a.rotate(a_attachment);
        let b_attachment = b.rotate(b_attachment);

        let delta_lambda = Self::delta_lambda(
            self.lambda(),
            self.magnitude(a_world_position, b_world_position),
            self.compliance(),
            [gradient, gradient],
            [a_attachment, b_attachment],
            [a, b],
            dt,
        );
        if delta_lambda.abs() <= std::f32::EPSILON {
            // Nothing will change, do nothing
            return;
        }

        // lambda += delta_lambda
        self.set_lambda(self.lambda() + delta_lambda);

        // Apply impulse
        let positional_impulse = gradient * delta_lambda;
        a.apply_positional_impulse(positional_impulse, a_attachment, 1.0);
        b.apply_positional_impulse(positional_impulse, b_attachment, -1.0);
    }
}

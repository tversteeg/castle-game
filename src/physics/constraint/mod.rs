pub mod distance;
pub mod penetration;

use slotmap::HopSlotMap;
use vek::Vec2;

use super::{RigidBody, RigidBodyKey};

/// XPBD constraint between one or more rigid bodies.
pub trait Constraint<const RIGIDBODIES: usize> {
    /// Current stored lambda.
    fn lambda(&self) -> f64;

    /// Set the lambda.
    fn set_lambda(&mut self, lambda: f64);

    /// Solve the constraint.
    ///
    /// Applies the force immediately to the rigidbodies.
    fn solve(&mut self, rigidbodies: &mut HopSlotMap<RigidBodyKey, RigidBody>, dt: f64);

    /// Array of indices for each rigidbody that the constraint needs.
    fn rigidbody_keys(&self) -> [RigidBodyKey; RIGIDBODIES];

    /// Reset the constraint at the beginning of a step (not a sub-step).
    fn reset(&mut self) {
        self.set_lambda(0.0);
    }

    /// Mutable references to all rigidbodies used in the constraint.
    fn rigidbodies<'a>(
        &self,
        rigidbodies: &'a HopSlotMap<RigidBodyKey, RigidBody>,
    ) -> [&'a RigidBody; RIGIDBODIES] {
        puffin::profile_scope!("Rigidbodies");

        self.rigidbody_keys().map(|key| {
            rigidbodies
                .get(key)
                .expect("Couldn't get rigidbody for contraint")
        })
    }

    /// Mutable references to all rigidbodies used in the constraint.
    fn rigidbodies_mut<'a>(
        &self,
        rigidbodies: &'a mut HopSlotMap<RigidBodyKey, RigidBody>,
    ) -> [&'a mut RigidBody; RIGIDBODIES] {
        puffin::profile_scope!("Rigidbodies mut");

        rigidbodies
            .get_disjoint_mut(self.rigidbody_keys())
            .expect("Couldn't get rigidbodies for constraint")
    }

    /// Calculate a generic form of the lambda update.
    fn delta_lambda<const AMOUNT: usize>(
        lambda: f64,
        magnitude: f64,
        compliance: f64,
        gradients: [Vec2<f64>; AMOUNT],
        attachments: [Vec2<f64>; AMOUNT],
        bodies: [&RigidBody; AMOUNT],
        dt: f64,
    ) -> f64 {
        puffin::profile_scope!("Delta lambda");

        let generalized_inverse_mass_sum: f64 = gradients
            .into_iter()
            .zip(attachments)
            .zip(bodies)
            .map(|((gradient, attachment), body)| {
                body.inverse_mass_at_relative_point(attachment, gradient)
            })
            .sum();

        if generalized_inverse_mass_sum <= std::f64::EPSILON {
            // Avoid divisions by zero
            return 0.0;
        }

        let stiffness = compliance / dt.powi(2);

        (-magnitude - stiffness * lambda) / (generalized_inverse_mass_sum + stiffness)
    }
}

/// Constraint specialization for position restrictions.
pub trait PositionalConstraint: Constraint<2> {
    /// Direction normalized vector.
    fn gradient(&self, a_world_position: Vec2<f64>, b_world_position: Vec2<f64>) -> Vec2<f64>;

    /// Magnitude.
    fn magnitude(&self, a_world_position: Vec2<f64>, b_world_position: Vec2<f64>) -> f64;

    /// Compliance.
    fn compliance(&self) -> f64;

    /// Calculate and apply the forces from the implemented methods.
    ///
    /// Updates the lambda.
    fn apply(
        &mut self,
        a: &mut RigidBody,
        a_attachment: Vec2<f64>,
        b: &mut RigidBody,
        b_attachment: Vec2<f64>,
        dt: f64,
    ) {
        puffin::profile_scope!("Apply positional constraint forces");

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
        if delta_lambda.abs() <= std::f64::EPSILON {
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

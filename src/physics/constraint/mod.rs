//pub mod distance;
pub mod penetration;

use hecs::{Bundle, Entity, World};
use vek::Vec2;

use crate::physics::rigidbody::{ApplyPositionalImpulse, InverseMassAtRelativePoint, LocalToWorld};

use super::{
    rigidbody::{Inertia, InvMass, Orientation, Position, Translation},
    RigidBody, RigidBodyKey,
};

/// XPBD constraint between one or more rigid bodies.
pub trait Constraint<const RIGIDBODIES: usize> {
    /// Current stored lambda.
    fn lambda(&self) -> f64;

    /// Set the lambda.
    fn set_lambda(&mut self, lambda: f64);

    /// Solve the constraint.
    ///
    /// Applies the force immediately to the rigidbodies.
    fn solve(&mut self, world: &mut World, dt: f64);

    /// Reset the constraint at the beginning of a step (not a sub-step).
    fn reset(&mut self) {
        self.set_lambda(0.0);
    }

    /// Calculate a generic form of the lambda update.
    fn delta_lambda<const AMOUNT: usize>(
        lambda: f64,
        magnitude: f64,
        compliance: f64,
        gradients: [Vec2<f64>; AMOUNT],
        attachments: [Vec2<f64>; AMOUNT],
        bodies: [(&InvMass, &Inertia); AMOUNT],
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
        a: Entity,
        a_attachment: Vec2<f64>,
        b: Entity,
        b_attachment: Vec2<f64>,
        world: &mut World,
        dt: f64,
    ) {
        puffin::profile_scope!("Apply positional constraint forces");

        // Used for querying the ECS
        #[derive(Bundle)]
        struct RigidBody {
            pos: Position,
            trans: Translation,
            rot: Orientation,
            inv_mass: InvMass,
            inertia: Inertia,
        }

        let a = world
            .query_one_mut::<&mut RigidBody>(a)
            .expect("Could not get rigidbody");
        let b = world
            .query_one_mut::<&mut RigidBody>(b)
            .expect("Could not get rigidbody");

        let a_world_position = (&a.pos, &a.trans, &a.rot).local_to_world(a_attachment);
        let b_world_position = (&b.pos, &b.trans, &b.rot).local_to_world(b_attachment);

        let gradient = self.gradient(a_world_position, b_world_position);

        // Rotate the attachments
        let a_attachment = a.rot.rotate(a_attachment);
        let b_attachment = b.rot.rotate(b_attachment);

        let delta_lambda = Self::delta_lambda(
            self.lambda(),
            self.magnitude(a_world_position, b_world_position),
            self.compliance(),
            [gradient, gradient],
            [a_attachment, b_attachment],
            [(&a.inv_mass, &a.inertia), (&b.inv_mass, &b.inertia)],
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
        (&mut a.trans, &mut a.rot, &a.inv_mass, &a.inertia).apply_positional_impulse(
            positional_impulse,
            a_attachment,
            1.0,
        );
        (&mut b.trans, &mut b.rot, &b.inv_mass, &b.inertia).apply_positional_impulse(
            positional_impulse,
            b_attachment,
            -1.0,
        );
    }
}

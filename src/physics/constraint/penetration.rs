use hecs::{View, World};
use vek::Vec2;

use crate::physics::{collision::CollisionResponse, rigidbody::RigidBodyQuery, RigidBodyKey};

use super::{Constraint, PositionalConstraint};

/// Short-lived collision constraint for resolving collisions.
#[derive(Debug, Clone)]
pub struct PenetrationConstraint {
    /// Object A.
    pub a: RigidBodyKey,
    /// Object B.
    pub b: RigidBodyKey,
    /// Collision response.
    pub response: CollisionResponse,
    /// Lambda value.
    ///
    /// Must be reset every frame.
    normal_lambda: f64,
    /// Normal lambda value.
    pub tangent_lambda: f64,
}

impl PenetrationConstraint {
    /// Constrain two rigidbodies with a spring so they can't be try to resolve the distance between them.
    ///
    /// RigidBodys must be indices.
    pub fn new(rigidbodies: [RigidBodyKey; 2], response: CollisionResponse) -> Self {
        let normal_lambda = 0.0;
        let tangent_lambda = 0.0;
        let [a, b] = rigidbodies;
        debug_assert_ne!(a, b);

        Self {
            normal_lambda,
            tangent_lambda,
            a,
            b,
            response,
        }
    }

    /// Local attachment for object A.
    #[inline]
    pub fn a_attachment(&self) -> Vec2<f64> {
        self.response.local_contact_1
    }

    /// Local attachment for object B.
    #[inline]
    pub fn b_attachment(&self) -> Vec2<f64> {
        self.response.local_contact_2
    }

    /// Calculate and apply friction between bodies.
    fn solve_friction(&mut self, a: &mut RigidBodyQuery, b: &mut RigidBodyQuery, dt: f64) {
        puffin::profile_scope!("Solve friction");

        // Rotate the attachments
        let a_attachment = a.rotate(self.a_attachment());
        let b_attachment = b.rotate(self.b_attachment());

        // Relative motion
        let a_delta_pos = a.relative_motion_at_point(a_attachment);
        let b_delta_pos = b.relative_motion_at_point(b_attachment);
        let delta_pos = a_delta_pos - b_delta_pos;

        let normal = self.response.normal;
        let delta_pos_tangent = delta_pos - delta_pos.dot(normal) * normal;

        // Relative tangential movement
        let (sliding_tangent, sliding_len) = delta_pos_tangent.normalized_and_get_magnitude();
        if sliding_len <= std::f64::EPSILON
            || sliding_len >= a.combine_static_frictions(b) * self.response.penetration
        {
            // Sliding is outside of the static zone, dynamic friction applies here
            return;
        }

        let tangent_delta_lambda = Self::delta_lambda(
            self.tangent_lambda,
            sliding_len,
            self.compliance(),
            [sliding_tangent, -sliding_tangent],
            [a_attachment, b_attachment],
            [
                (a.inv_mass.clone(), a.inertia.clone()),
                (b.inv_mass.clone(), b.inertia.clone()),
            ],
            dt,
        );
        if tangent_delta_lambda.abs() <= std::f64::EPSILON {
            // Nothing will change, do nothing
            return;
        }
        self.tangent_lambda += tangent_delta_lambda;

        // Apply impulse
        let positional_impulse = sliding_tangent * tangent_delta_lambda;
        a.apply_positional_impulse(positional_impulse, a_attachment, 1.0);
        b.apply_positional_impulse(positional_impulse, b_attachment, -1.0);
    }

    /// Calculate and apply contact velocities after solve step.
    pub fn solve_velocities(&self, rigidbodies: &mut View<RigidBodyQuery>, dt: f64) {
        puffin::profile_scope!("Solve velocities");

        if self.lambda().abs() <= std::f64::EPSILON {
            // Nothing happened in this constraint
            return;
        }

        // Query the rigidbodies
        let [mut a, mut b] = rigidbodies
            .get_mut_n([self.a, self.b])
            .map(|v| v.expect("Rigidbody not found"));

        // Collisions between static objects shouldn't be added
        debug_assert!(!a.is_static() || !b.is_static());

        // Rotate the attachments
        let a_attachment = a.rotate(self.a_attachment());
        let b_attachment = b.rotate(self.b_attachment());

        let normal = self.response.normal;
        let a_prev_contact_vel = a.prev_contact_velocity(a_attachment).unwrap_or_default();
        let b_prev_contact_vel = b.prev_contact_velocity(b_attachment).unwrap_or_default();
        let prev_rel_contact_vel = a_prev_contact_vel - b_prev_contact_vel;

        let prev_normal_vel = normal.dot(prev_rel_contact_vel);

        // Different velocities
        let a_contact_vel = a.contact_velocity(a_attachment).unwrap_or_default();
        let b_contact_vel = b.contact_velocity(b_attachment).unwrap_or_default();
        let rel_contact_vel = a_contact_vel - b_contact_vel;

        let normal_vel = normal.dot(rel_contact_vel);
        let tangent_vel = rel_contact_vel - normal * normal_vel;
        let tangent_vel_magnitude = tangent_vel.magnitude();

        // Dynamic friction
        let dynamic_friction_impulse = if tangent_vel_magnitude <= std::f64::EPSILON {
            Vec2::zero()
        } else {
            let normal_impulse = self.normal_lambda / dt;

            // Friction can never exceed the velocity itself
            -tangent_vel
                * (a.combine_dynamic_frictions(&b) * normal_impulse.abs() / tangent_vel_magnitude)
                    .min(1.0)
        };

        // Restitution
        let restitution_coefficient = if normal_vel.abs() <= 2.0 * dt {
            // Prevent some jittering
            0.0
        } else {
            a.combine_restitutions(&b)
        };

        let restitution_impulse =
            normal * (-normal_vel + (-restitution_coefficient * prev_normal_vel).min(0.0));

        // Calcule the new velocity
        let delta_vel = dynamic_friction_impulse + restitution_impulse;
        let (delta_vel_normal, delta_vel_magnitude) = delta_vel.normalized_and_get_magnitude();
        if delta_vel_magnitude <= std::f64::EPSILON {
            return;
        }

        let a_generalized_inverse_mass =
            a.inverse_mass_at_relative_point(a_attachment, delta_vel_normal);
        let b_generalized_inverse_mass =
            b.inverse_mass_at_relative_point(b_attachment, delta_vel_normal);
        let generalized_inverse_mass_sum = a_generalized_inverse_mass + b_generalized_inverse_mass;
        if generalized_inverse_mass_sum <= std::f64::EPSILON {
            // Avoid divisions by zero
            return;
        }

        // Apply velocity impulses and updates
        let velocity_impulse = delta_vel / generalized_inverse_mass_sum;
        a.apply_velocity_impulse(velocity_impulse, a_attachment, 1.0);
        b.apply_velocity_impulse(velocity_impulse, b_attachment, -1.0);
    }
}

impl Constraint<2> for PenetrationConstraint {
    fn solve(&mut self, rigidbodies: &mut View<RigidBodyQuery>, dt: f64) {
        puffin::profile_scope!("Solve penetration constraint");

        if self.response.penetration <= std::f64::EPSILON {
            // Ignore fake collisions
            return;
        }

        // Query the rigidbodies
        let [mut a, mut b] = rigidbodies
            .get_mut_n([self.a, self.b])
            .map(|v| v.expect("Rigidbody not found"));

        // Apply the regular positional constraint to resolve overlapping
        self.apply(&mut a, self.a_attachment(), &mut b, self.b_attachment(), dt);

        // Apply an additional friction check
        self.solve_friction(&mut a, &mut b, dt);
    }

    #[inline]
    fn lambda(&self) -> f64 {
        self.normal_lambda
    }

    #[inline]
    fn set_lambda(&mut self, lambda: f64) {
        self.normal_lambda = lambda;
    }

    /// Override for the other lambdas.
    #[inline]
    fn reset(&mut self) {
        self.normal_lambda = 0.0;
        self.tangent_lambda = 0.0;
    }
}

impl PositionalConstraint for PenetrationConstraint {
    #[inline]
    fn gradient(&self, _a_global_position: Vec2<f64>, _b_global_position: Vec2<f64>) -> Vec2<f64> {
        self.response.normal
    }

    #[inline]
    fn magnitude(&self, _a_global_position: Vec2<f64>, _b_global_position: Vec2<f64>) -> f64 {
        self.response.penetration
    }

    #[inline]
    fn compliance(&self) -> f64 {
        0.00001
    }
}

use hecs::{Bundle, World};
use slotmap::HopSlotMap;
use vek::Vec2;

use crate::physics::{
    collision::CollisionResponse,
    rigidbody::{
        AngularVelocity, ApplyPositionalImpulse, ApplyVelocityImpulse, ContactVelocity, Friction,
        Inertia, InvMass, InverseMassAtRelativePoint, Orientation, Position, PrevAngularVelocity,
        PrevOrientation, PrevPosition, PrevVelocity, RelativeMotionAtPoint, Restitution, RigidBody,
        Translation, Velocity,
    },
    Physics, RigidBodyKey,
};

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

        Self {
            normal_lambda,
            tangent_lambda,
            a,
            b,
            response,
        }
    }

    /// Local attachment for object A.
    pub fn a_attachment(&self) -> Vec2<f64> {
        self.response.local_contact_1
    }

    /// Local attachment for object B.
    pub fn b_attachment(&self) -> Vec2<f64> {
        self.response.local_contact_2
    }

    /// Calculate and apply friction between bodies.
    pub fn solve_friction(&mut self, world: &mut World, dt: f64) {
        puffin::profile_scope!("Solve friction");

        // Used for querying the ECS
        #[derive(Bundle)]
        struct RigidBody {
            pos: Position,
            prev_pos: PrevPosition,
            trans: Translation,
            rot: Orientation,
            prev_rot: PrevOrientation,
            friction: Friction,
            inv_mass: InvMass,
            inertia: Inertia,
        }

        let a = world
            .query_one_mut::<&mut RigidBody>(self.a)
            .expect("Could not get rigidbody");
        let b = world
            .query_one_mut::<&mut RigidBody>(self.b)
            .expect("Could not get rigidbody");

        // Rotate the attachments
        let a_attachment = a.rot.rotate(self.a_attachment());
        let b_attachment = b.rot.rotate(self.b_attachment());

        // Relative motion
        let a_delta_pos =
            (&a.pos, &a.prev_pos, &a.trans, &a.prev_rot).relative_motion_at_point(a_attachment);
        let b_delta_pos =
            (&b.pos, &b.prev_pos, &b.trans, &b.prev_rot).relative_motion_at_point(b_attachment);
        let delta_pos = a_delta_pos - b_delta_pos;

        let normal = self.response.normal;
        let delta_pos_tangent = delta_pos - delta_pos.dot(normal) * normal;

        // Relative tangential movement
        let (sliding_tangent, sliding_len) = delta_pos_tangent.normalized_and_get_magnitude();
        if sliding_len <= std::f64::EPSILON
            || sliding_len
                >= a.friction.combine_static_frictions(&b.friction) * self.response.penetration
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
            [(&a.inv_mass, &a.inertia), (&b.inv_mass, &b.inertia)],
            dt,
        );
        if tangent_delta_lambda.abs() <= std::f64::EPSILON {
            // Nothing will change, do nothing
            return;
        }
        self.tangent_lambda += tangent_delta_lambda;

        // Apply impulse
        let positional_impulse = sliding_tangent * tangent_delta_lambda;
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

    /// Calculate and apply contact velocities after solve step.
    pub fn solve_velocities(&self, world: &mut World, dt: f64) {
        puffin::profile_scope!("Solve velocities");

        if self.lambda().abs() <= std::f64::EPSILON {
            // Nothing happened in this constraint
            return;
        }

        // Used for querying the ECS
        #[derive(Bundle)]
        struct RigidBody {
            pos: Position,
            prev_pos: PrevPosition,
            trans: Translation,
            vel: Velocity,
            prev_vel: PrevVelocity,
            rot: Orientation,
            prev_rot: PrevOrientation,
            ang_vel: AngularVelocity,
            prev_ang_vel: PrevAngularVelocity,
            friction: Friction,
            inv_mass: InvMass,
            inertia: Inertia,
            restitution: Restitution,
        }

        let a = world
            .query_one_mut::<&mut RigidBody>(self.a)
            .expect("Could not get rigidbody");
        let b = world
            .query_one_mut::<&mut RigidBody>(self.b)
            .expect("Could not get rigidbody");

        // Rotate the attachments
        let a_attachment = a.rot.rotate(self.a_attachment());
        let b_attachment = b.rot.rotate(self.b_attachment());

        let normal = self.response.normal;
        let a_prev_contact_vel = (&a.prev_vel, &a.prev_ang_vel).contact_velocity(a_attachment);
        let b_prev_contact_vel = (&b.prev_vel, &b.prev_ang_vel).contact_velocity(b_attachment);
        let prev_rel_contact_vel = a_prev_contact_vel - b_prev_contact_vel;

        let prev_normal_vel = normal.dot(prev_rel_contact_vel);

        // Different velocities
        let a_contact_vel = (&a.vel, &a.ang_vel).contact_velocity(a_attachment);
        let b_contact_vel = (&b.vel, &b.ang_vel).contact_velocity(b_attachment);
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
                * (a.friction.combine_dynamic_frictions(&b.friction) * normal_impulse.abs()
                    / tangent_vel_magnitude)
                    .min(1.0)
        };

        // Restitution
        let restitution_coefficient = if normal_vel.abs() <= 2.0 * dt {
            // Prevent some jittering
            0.0
        } else {
            a.restitution.combine_restitutions(&b.restitution)
        };

        let restitution_impulse =
            normal * (-normal_vel + (-restitution_coefficient * prev_normal_vel).min(0.0));

        // Calcule the new velocity
        let delta_vel = dynamic_friction_impulse + restitution_impulse;
        let (delta_vel_normal, delta_vel_magnitude) = delta_vel.normalized_and_get_magnitude();
        if delta_vel_magnitude <= std::f64::EPSILON {
            return;
        }

        let a_generalized_inverse_mass = (&a.inv_mass, &a.inertia)
            .inverse_mass_at_relative_point(a_attachment, delta_vel_normal);
        let b_generalized_inverse_mass = (&b.inv_mass, &b.inertia)
            .inverse_mass_at_relative_point(b_attachment, delta_vel_normal);
        let generalized_inverse_mass_sum = a_generalized_inverse_mass + b_generalized_inverse_mass;
        if generalized_inverse_mass_sum <= std::f64::EPSILON {
            // Avoid divisions by zero
            return;
        }

        // Apply velocity impulses and updates
        let velocity_impulse = delta_vel / generalized_inverse_mass_sum;
        (&mut a.vel, &mut a.ang_vel, &a.inv_mass, &a.inertia).apply_velocity_impulse(
            velocity_impulse,
            a_attachment,
            1.0,
        );
        (&mut b.vel, &mut b.ang_vel, &b.inv_mass, &b.inertia).apply_velocity_impulse(
            velocity_impulse,
            b_attachment,
            -1.0,
        );
    }
}

impl Constraint<2> for PenetrationConstraint {
    fn solve(&mut self, world: &mut World, dt: f64) {
        puffin::profile_scope!("Solve penetration constraint");

        if self.response.penetration <= std::f64::EPSILON {
            // Ignore fake collisions
            return;
        }

        // Apply the regular positional constraint to resolve overlapping
        self.apply(
            self.a,
            self.a_attachment(),
            self.b,
            self.b_attachment(),
            world,
            dt,
        );

        // Apply an additional friction check
        self.solve_friction(world, dt);
    }

    fn lambda(&self) -> f64 {
        self.normal_lambda
    }

    fn set_lambda(&mut self, lambda: f64) {
        self.normal_lambda = lambda;
    }

    /// Override for the other lambdas.
    fn reset(&mut self) {
        self.normal_lambda = 0.0;
        self.tangent_lambda = 0.0;
    }
}

impl PositionalConstraint for PenetrationConstraint {
    fn gradient(&self, _a_global_position: Vec2<f64>, _b_global_position: Vec2<f64>) -> Vec2<f64> {
        self.response.normal
    }

    fn magnitude(&self, _a_global_position: Vec2<f64>, _b_global_position: Vec2<f64>) -> f64 {
        self.response.penetration
    }

    fn compliance(&self) -> f64 {
        0.00001
    }
}

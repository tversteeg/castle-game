use std::fmt::Display;

use smallvec::SmallVec;
use vek::{Aabr, Vec2};

use crate::math::{Iso, Rotation};

use super::collision::{shape::Shape, CollisionResponse};

/// How far away we predict the impulses to move us for checking the collision during the next full deltatime.
const PREDICTED_POSITION_MULTIPLIER: f32 = 2.0;

/// Rigidbody index type.
pub type RigidBodyIndex = u32;

/// Represents any physics object that can have constraints applied.
#[derive(Clone, Default)]
pub struct RigidBody {
    /// Global position.
    pos: Vec2<f32>,
    /// Previous position.
    prev_pos: Vec2<f32>,
    /// Global offset that will be added to the body.
    translation: Vec2<f32>,
    /// Velocity.
    vel: Vec2<f32>,
    /// Previous velocity.
    prev_vel: Vec2<f32>,
    /// Orientation in radians.
    rot: Rotation,
    /// Previous orientation.
    prev_rot: Rotation,
    /// Angular velocity.
    ang_vel: f32,
    /// Previous angular velocity.
    prev_ang_vel: f32,
    /// Inertia tensor, corresponds to mass in rotational terms.
    ///
    /// Torque needed for an angular acceleration.
    inertia: f32,
    /// Linear damping.
    lin_damping: f32,
    /// Angular damping.
    ang_damping: f32,
    /// External forces.
    ext_force: Vec2<f32>,
    // External torque.
    ext_torque: f32,
    /// Inverse of the mass.
    inv_mass: f32,
    /// Friction coefficient, for now there's no difference between dynamic and static friction.
    friction: f32,
    /// Restitution coefficient, how bouncy collisions are.
    restitution: f32,
    /// Collision shape.
    shape: Shape,
    /// If > 0 we are sleeping, which means we don't have to calculate a lot of steps.
    ///
    /// After a certain time the velocity and position will be set to zero.
    time_sleeping: f32,
}

impl RigidBody {
    /// Construct a new rigidbody without movements.
    ///
    /// Gravity is applied as an external force.
    pub fn new<S>(pos: Vec2<f32>, shape: S) -> Self
    where
        S: Into<Shape>,
    {
        let settings = crate::settings();

        Self::new_external_force(
            pos,
            Vec2::new(0.0, settings.physics.gravity),
            settings.physics.air_friction,
            settings.physics.rotation_friction,
            1.0,
            shape,
        )
    }

    /// Construct a new rigidbody with acceleration.
    pub fn new_external_force<S>(
        pos: Vec2<f32>,
        ext_force: Vec2<f32>,
        lin_damping: f32,
        ang_damping: f32,
        density: f32,
        shape: S,
    ) -> Self
    where
        S: Into<Shape>,
    {
        let prev_pos = pos;
        let vel = Vec2::default();
        let prev_vel = vel;
        let ang_vel = 0.0;
        let prev_ang_vel = ang_vel;
        let rot = Rotation::default();
        let prev_rot = rot;
        let ext_torque = 0.0;
        let friction = 0.3;
        let restitution = 0.3;
        let translation = Vec2::zero();
        let time_sleeping = 0.0;
        let shape = shape.into();
        let mass_properties = shape.mass_properties(density);
        let inv_mass = mass_properties.mass().recip();
        let inertia = mass_properties.principal_inertia();

        Self {
            pos,
            prev_pos,
            translation,
            vel,
            prev_vel,
            ext_force,
            ext_torque,
            lin_damping,
            inv_mass,
            inertia,
            rot,
            prev_rot,
            ang_vel,
            prev_ang_vel,
            ang_damping,
            shape,
            friction,
            restitution,
            time_sleeping,
        }
    }

    /// Construct a fixed rigidbody with infinite mass and no gravity.
    pub fn new_fixed<S>(pos: Vec2<f32>, shape: S) -> Self
    where
        S: Into<Shape>,
    {
        let inv_mass = 0.0;

        let prev_pos = pos;
        let translation = Vec2::zero();
        let vel = Vec2::zero();
        let prev_vel = vel;
        let rot = Rotation::default();
        let prev_rot = Rotation::default();
        let ang_vel = 0.0;
        let prev_ang_vel = ang_vel;
        let lin_damping = 0.0;
        let ang_damping = 0.0;
        let ext_force = Vec2::zero();
        let ext_torque = 0.0;
        let friction = 0.5;
        let restitution = 0.2;
        let time_sleeping = 0.0;
        let shape = shape.into();
        let inertia = 1.0;

        Self {
            pos,
            shape,
            prev_pos,
            translation,
            prev_vel,
            vel,
            rot,
            prev_rot,
            ang_vel,
            prev_ang_vel,
            inertia,
            lin_damping,
            ang_damping,
            ext_force,
            ext_torque,
            inv_mass,
            friction,
            restitution,
            time_sleeping,
        }
    }

    /// Apply velocity after creating a rigidbody.
    pub fn with_velocity(mut self, velocity: Vec2<f32>) -> Self {
        self.vel = velocity;
        self.prev_vel = velocity;

        self
    }

    /// Set the density.
    ///
    /// This will change the mass and inertia.
    pub fn with_density(mut self, density: f32) -> Self {
        let mass_properties = self.shape.mass_properties(density);
        self.inv_mass = mass_properties.mass().recip();
        self.inertia = mass_properties.principal_inertia();

        self
    }

    /// Set the dynamic and static frictions.
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction;

        self
    }

    /// Set the restitution.
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution;

        self
    }

    /// Set the linear damping.
    pub fn with_linear_damping(mut self, linear_damping: f32) -> Self {
        self.lin_damping = linear_damping;

        self
    }

    /// Set the angular damping.
    pub fn with_angular_damping(mut self, angular_damping: f32) -> Self {
        self.ang_damping = angular_damping;

        self
    }

    /// Perform a single (sub-)step with a deltatime.
    pub fn integrate(&mut self, dt: f32) {
        if !self.is_active() {
            return;
        }

        // Position update
        self.prev_pos = self.pos;

        // Apply damping if applicable
        if self.lin_damping != 1.0 {
            self.vel *= 1.0 / (1.0 + dt * self.lin_damping);
        }

        // Apply external forces
        self.vel += (dt * self.ext_force) / self.inv_mass.recip();
        self.translation += dt * self.vel;

        // Rotation update
        self.prev_rot = self.rot;

        // Apply damping if applicable
        if self.ang_damping != 1.0 {
            self.ang_vel *= 1.0 / (1.0 + dt * self.ang_damping);
        }

        self.ang_vel += dt * self.inverse_inertia() * self.ext_torque;
        self.rot += dt * self.ang_vel;
    }

    /// Add velocities.
    pub fn update_velocities(&mut self, dt: f32) {
        self.prev_vel = self.vel;
        self.vel = (self.pos - self.prev_pos + self.translation) / dt;

        self.prev_ang_vel = self.ang_vel;
        self.ang_vel = (self.rot - self.prev_rot).to_radians() / dt;
    }

    /// Apply translations to the position.
    pub fn apply_translation(&mut self) {
        if !self.is_active() {
            return;
        }

        self.pos += self.translation;
        self.translation = Vec2::zero();
    }

    /// Apply a force by moving the position, which will trigger velocity increments.
    pub fn apply_force(&mut self, force: Vec2<f32>) {
        self.translation += force;
    }

    /// Apply a rotational force in radians.
    pub fn apply_rotational_force(&mut self, force: f32) {
        self.rot += force;
    }

    /// Apply torque from an external source.
    pub fn apply_torque(&mut self, torque: f32) {
        self.ext_torque += torque;
    }

    /// Set global position.
    pub fn set_position(&mut self, pos: Vec2<f32>, force: bool) {
        self.pos = pos;
        if !force {
            self.prev_pos = pos;
            self.translation = Vec2::zero();
            self.vel = Vec2::zero();
        }
    }

    /// Set the rigidbody to sleeping if the velocities are below the treshold.
    pub fn mark_sleeping(&mut self, dt: f32) {
        if self.is_static() {
            return;
        }

        // TODO: make these values configurable
        if self.vel.magnitude_squared() > 1.0 || self.ang_vel.abs() > 1.0 {
            self.time_sleeping = 0.0;
        } else if self.time_sleeping < 0.5 {
            self.time_sleeping += dt;
        } else {
            // Set the velocities to zero to prevent jittering
            self.vel = Vec2::zero();
            self.ang_vel = 0.0;
        }
    }

    /// Global position.
    pub fn position(&self) -> Vec2<f32> {
        self.pos + self.translation
    }

    /// Global linear velocity.
    pub fn velocity(&self) -> Vec2<f32> {
        self.vel
    }

    /// Global angular velocity.
    pub fn angular_velocity(&self) -> f32 {
        self.ang_vel
    }

    /// Global position with rotation.
    pub fn iso(&self) -> Iso {
        Iso::new(self.position(), self.rot)
    }

    /// Orientation of the body.
    pub fn rotation(&self) -> Rotation {
        self.rot
    }

    /// Calculate generalized inverse mass at a relative point along the normal vector.
    pub fn inverse_mass_at_relative_point(&self, point: Vec2<f32>, normal: Vec2<f32>) -> f32 {
        if self.is_static() {
            return 0.0;
        }

        // Perpendicular dot product of `point` with `normal`
        let perp_dot = (point.x * normal.y) - (point.y * normal.x);

        self.inv_mass + self.inverse_inertia() * perp_dot.powi(2)
    }

    /// Calculate the update in rotation when a position change is applied at a specific point.
    pub fn delta_rotation_at_point(&self, point: Vec2<f32>, impulse: Vec2<f32>) -> f32 {
        // Perpendicular dot product of `point` with `impulse`
        let perp_dot = (point.x * impulse.y) - (point.y * impulse.x);

        self.inverse_inertia() * perp_dot
    }

    /// Apply a positional impulse at a point.
    ///
    // TODO: can we remove the sign by directly negating the impulse?
    pub fn apply_positional_impulse(
        &mut self,
        positional_impulse: Vec2<f32>,
        point: Vec2<f32>,
        sign: f32,
    ) {
        if self.is_static() {
            // Ignore when we're a static body
            return;
        }

        self.apply_force(sign * positional_impulse * self.inv_mass);

        // Change rotation of body
        self.apply_rotational_force(sign * self.delta_rotation_at_point(point, positional_impulse));
    }

    /// Apply a velocity change at a point.
    pub fn apply_velocity_impulse(
        &mut self,
        velocity_impulse: Vec2<f32>,
        point: Vec2<f32>,
        sign: f32,
    ) {
        if self.is_static() {
            // Ignore when we're a static body
            return;
        }

        self.vel += sign * velocity_impulse * self.inv_mass;
        self.ang_vel += sign * self.delta_rotation_at_point(point, velocity_impulse);
    }

    /// Calculate the contact velocity based on a local relative rotated point.
    pub fn contact_velocity(&self, point: Vec2<f32>) -> Vec2<f32> {
        // Perpendicular
        let perp = Vec2::new(-point.y, point.x);

        self.vel + self.ang_vel * perp
    }

    /// Calculate the contact velocity based on a local relative rotated point with the previous velocities.
    pub fn prev_contact_velocity(&self, point: Vec2<f32>) -> Vec2<f32> {
        // Perpendicular
        let perp = Vec2::new(-point.y, point.x);

        self.prev_vel + self.prev_ang_vel * perp
    }

    /// Delta position of a point.
    pub fn relative_motion_at_point(&self, point: Vec2<f32>) -> Vec2<f32> {
        self.pos - self.prev_pos + self.translation + point - self.prev_rot.rotate(point)
    }

    /// Inverse of the inertia tensor.
    pub fn inverse_inertia(&self) -> f32 {
        self.inertia.recip()
    }

    /// Axis-aligned bounding rectangle.
    pub fn aabr(&self) -> Aabr<f32> {
        self.shape.aabr(self.iso())
    }

    /// Vertices for the body.
    pub fn vertices(&self) -> Vec<Vec2<f32>> {
        self.shape.vertices(self.iso())
    }

    /// Axis-aligned bounding rectangle with a predicted future position added.
    ///
    /// WARNING: `dt` is not from the substep but from the full physics step.
    pub fn predicted_aabr(&self, dt: f32) -> Aabr<f32> {
        // Start with the future aabr
        let mut aabr = self.shape.aabr(Iso::new(
            self.position() + self.vel * PREDICTED_POSITION_MULTIPLIER * dt,
            self.rot,
        ));

        // Add the current aabr
        aabr.expand_to_contain(self.aabr());

        aabr
    }

    /// Check if it collides with another rigidbody.
    pub fn collides(&self, other: &RigidBody) -> SmallVec<[CollisionResponse; 4]> {
        self.shape.collides(self.iso(), &other.shape, other.iso())
    }

    /// Rotate a point in local space.
    pub fn rotate(&self, point: Vec2<f32>) -> Vec2<f32> {
        self.rot.rotate(point)
    }

    /// Calculate the world position of a relative point on this body without rotation in mind.
    pub fn local_to_world(&self, point: Vec2<f32>) -> Vec2<f32> {
        // Then translate it to the position
        self.position() + self.rotate(point)
    }

    /// Whether this rigidbody doesn't move and has infinite mass.
    pub fn is_static(&self) -> bool {
        self.inv_mass == 0.0
    }

    /// Whether the rigidbody is in a sleeping state.
    pub fn is_sleeping(&self) -> bool {
        self.time_sleeping >= 0.5
    }

    /// Whether this is an active rigid body, means it's not sleeping and not static.
    pub fn is_active(&self) -> bool {
        !self.is_static() && !self.is_sleeping()
    }

    /// Friction that needs to be overcome before resting objects start sliding.
    pub fn static_friction(&self) -> f32 {
        self.friction
    }

    /// Friction that's applied to dynamic moving object after static friction has been overcome.
    pub fn dynamic_friction(&self) -> f32 {
        self.friction
    }

    /// Combine the static frictions between this and another body.
    pub fn combine_static_frictions(&self, other: &Self) -> f32 {
        (self.static_friction() + other.static_friction()) / 2.0
    }

    /// Combine the dynamic frictions between this and another body.
    pub fn combine_dynamic_frictions(&self, other: &Self) -> f32 {
        (self.dynamic_friction() + other.dynamic_friction()) / 2.0
    }

    /// Combine the restitutions between this and another body.
    pub fn combine_restitutions(&self, other: &Self) -> f32 {
        (self.restitution + other.restitution) / 2.0
    }

    /// Current direction the body is moving in.
    pub fn direction(&self) -> Vec2<f32> {
        (self.pos - self.prev_pos).normalized()
    }
}

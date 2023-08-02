use std::fmt::Display;

use arrayvec::ArrayVec;
use vek::{Aabr, Vec2};

use crate::{math::Rotation, SIZE};

use super::collision::{shape::Rectangle, CollisionResponse, NarrowCollision};

/// How far away we predict the impulses to move us for checking the collision during the next full deltatime.
const PREDICTED_POSITION_MULTIPLIER: f32 = 2.0;

/// Rigidbody index type.
pub type RigidBodyIndex = u32;

/// Represents any physics object that can have constraints applied.
#[derive(Debug, Clone, Default)]
pub struct RigidBody {
    /// Global position.
    pos: Vec2<f32>,
    /// Previous position.
    prev_pos: Vec2<f32>,
    /// Velocity.
    vel: Vec2<f32>,
    /// Orientation in radians.
    rot: Rotation,
    /// Previous orientation.
    prev_rot: Rotation,
    /// Angular velocity.
    ang_vel: f32,
    /// Inertia tensor, corresponds to mass in rotational terms.
    ///
    /// Torque needed for an angulare acceleration.
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
    /// Collision shape.
    shape: Rectangle,
}

impl RigidBody {
    /// Construct a new rigidbody without movements.
    ///
    /// Gravity is applied as an external force.
    pub fn new(pos: Vec2<f32>, mass: f32, shape: Rectangle) -> Self {
        let settings = crate::settings();

        Self::with_external_force(
            pos,
            Vec2::new(0.0, settings.physics.gravity),
            settings.physics.air_friction,
            mass,
            shape,
        )
    }

    /// Construct a new rigidbody with acceleration.
    pub fn with_external_force(
        pos: Vec2<f32>,
        ext_force: Vec2<f32>,
        damping: f32,
        mass: f32,
        shape: Rectangle,
    ) -> Self {
        let inv_mass = mass.recip();
        let prev_pos = pos;
        let vel = Vec2::default();
        let ang_vel = 0.0;
        let lin_damping = damping;
        let ang_damping = damping;
        let rot = Rotation::default();
        let prev_rot = rot;
        let ext_torque = 0.0;

        // https://en.wikipedia.org/wiki/List_of_moments_of_inertia
        let inertia = mass * (shape.width().powi(2) + shape.height().powi(2)) / 12.0;

        Self {
            vel,
            pos,
            prev_pos,
            ext_force,
            ext_torque,
            lin_damping,
            inv_mass,
            inertia,
            rot,
            prev_rot,
            ang_vel,
            ang_damping,
            shape,
        }
    }

    /// Construct a fixed rigidbody with infinite mass and no gravity.
    pub fn fixed(pos: Vec2<f32>, shape: Rectangle) -> Self {
        let inv_mass = 0.0;

        let prev_pos = pos;
        let vel = Vec2::zero();
        let rot = Rotation::default();
        let prev_rot = Rotation::default();
        let ang_vel = 0.0;
        let inertia = 0.0;
        let lin_damping = 0.0;
        let ang_damping = 0.0;
        let ext_force = Vec2::zero();
        let ext_torque = 0.0;

        Self {
            pos,
            shape,
            prev_pos,
            vel,
            rot,
            prev_rot,
            ang_vel,
            inertia,
            lin_damping,
            ang_damping,
            ext_force,
            ext_torque,
            inv_mass,
        }
    }

    /// Perform a single (sub-)step with a deltatime.
    pub fn integrate(&mut self, dt: f32) {
        if self.inv_mass == 0.0 {
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
        self.pos += dt * self.vel;

        // Rotation update
        self.prev_rot = self.rot;

        // Apply damping if applicable
        if self.ang_damping != 1.0 {
            self.ang_vel *= 1.0 / (1.0 + dt * self.ang_damping);
        }

        self.ang_vel += dt * self.inverse_inertia() * self.ext_torque;
        self.rot += dt * self.ang_vel;
    }

    /// Last part of a single (sub-)step.
    pub fn solve(&mut self, dt: f32) {
        self.vel = (self.pos - self.prev_pos) / dt;

        self.ang_vel = (self.rot - self.prev_rot).to_radians() / dt;
        self.ang_vel = self.ang_vel.clamp(-0.1, 0.1);

        self.pos.y = self.pos.y.min(SIZE.h as f32);
    }

    /// Apply a force by moving the position, which will trigger velocity increments.
    pub fn apply_force(&mut self, force: Vec2<f32>) {
        self.pos += force;
    }

    /// Apply a rotational force in radians.
    pub fn apply_rotational_force(&mut self, force: f32) {
        self.rot += force;
    }

    /// Set global position.
    pub fn set_position(&mut self, pos: Vec2<f32>, force: bool) {
        self.pos = pos;
        if !force {
            self.prev_pos = pos;
            self.vel = Vec2::zero();
        }
    }

    /// Global position.
    pub fn position(&self) -> Vec2<f32> {
        self.pos
    }

    /// Rotation in radians.
    pub fn rotation(&self) -> f32 {
        self.rot.to_radians()
    }

    /// Calculate generalized inverse mass at a relative point along the normal vector.
    pub fn inverse_mass_at_relative_point(&self, point: Vec2<f32>, normal: Vec2<f32>) -> f32 {
        // Perpendicular dot product of `point` with `normal`
        let perp_dot = (point.x * normal.y) - (point.y * normal.x);

        self.inv_mass + self.inverse_inertia() * perp_dot.powi(2)
    }

    /// Calculate the update in rotation when a position change is applied at a specific point.
    pub fn delta_rotation_at_point(&self, point: Vec2<f32>, impulse: Vec2<f32>) -> f32 {
        // Perpendicular dot product of `point` with `normal`
        let perp_dot = (point.x * impulse.y) - (point.y * impulse.x);

        self.inverse_inertia() * perp_dot
    }

    /// Inverse of the mass.
    pub fn inverse_mass(&self) -> f32 {
        self.inv_mass
    }

    /// Inertia tensor, corresponds to mass in rotational terms.
    pub fn inertia(&self) -> f32 {
        self.inertia
    }

    /// Inverse of the inertia tensor.
    pub fn inverse_inertia(&self) -> f32 {
        self.inertia.recip()
    }

    /// Axis-aligned bounding rectangle.
    pub fn aabr(&self) -> Aabr<f32> {
        self.shape.aabr(self.pos, self.rot.to_radians())
    }

    /// Axis-aligned bounding rectangle with a predicted future position added.
    ///
    /// WARNING: `dt` is not from the substep but from the full physics step.
    pub fn predicted_aabr(&self, dt: f32) -> Aabr<f32> {
        // Start with the future aabr
        let mut aabr = self.shape.aabr(
            self.pos + self.vel * PREDICTED_POSITION_MULTIPLIER * dt,
            self.rot.to_radians(),
        );

        // Add the current aabr
        aabr.expand_to_contain(self.aabr());

        aabr
    }

    /// Check if it collides with another rigidbody.
    pub fn collides(&self, other: &RigidBody) -> ArrayVec<CollisionResponse, 2> {
        self.shape
            .collide_rectangle(self.pos, self.rot, other.shape, other.pos, other.rot)
    }

    /// Rotate a point in local space.
    pub fn rotate(&self, point: Vec2<f32>) -> Vec2<f32> {
        self.rot.rotate(point)
    }

    /// Calculate the world position of a relative point on this body without rotation in mind.
    pub fn local_to_world(&self, point: Vec2<f32>) -> Vec2<f32> {
        // Then translate it to the position
        self.pos + self.rotate(point)
    }
}

impl Display for RigidBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "vel: ({}, {})", self.vel.x.round(), self.vel.y.round())?;
        writeln!(f, "ang_vel: {}", self.ang_vel.round())?;

        Ok(())
    }
}

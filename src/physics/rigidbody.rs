use vek::Vec2;

use crate::assets::Assets;

/// Rigidbody index type.
pub type RigidBodyIndex = u32;

/// Represents any physics object that can have constraints applied.
#[derive(Debug, Clone)]
pub struct RigidBody {
    /// Global position.
    pos: Vec2<f64>,
    /// Previous position.
    prev_pos: Vec2<f64>,
    /// Velocity.
    vel: Vec2<f64>,
    /// Orientation in radians.
    rot: f64,
    /// Previous orientation.
    prev_rot: f64,
    /// Angular velocity.
    ang_vel: f64,
    /// Inertia tensor, corresponds to mass in rotational terms.
    inertia: f64,
    /// External forces.
    ext_force: Vec2<f64>,
    // External torque.
    ext_torque: f64,
    /// Inverse of the mass.
    inv_mass: f64,
}

impl RigidBody {
    /// Construct a new rigidbody without movements.
    ///
    /// Gravity is applied as an external force.
    pub fn new(pos: Vec2<f64>, mass: f64, assets: &Assets) -> Self {
        Self::with_external_force(pos, Vec2::new(0.0, assets.settings().physics.gravity), mass)
    }

    /// Construct a new rigidbody with acceleration.
    pub fn with_external_force(pos: Vec2<f64>, ext_force: Vec2<f64>, mass: f64) -> Self {
        let inv_mass = mass.recip();
        let prev_pos = pos;
        let vel = Vec2::default();
        let inertia = 1.0;
        let ang_vel = 0.0;
        let rot = 0.0;
        let prev_rot = 0.0;
        let ext_torque = 0.0;

        Self {
            vel,
            pos,
            prev_pos,
            ext_force,
            ext_torque,
            inv_mass,
            inertia,
            rot,
            prev_rot,
            ang_vel,
        }
    }

    /// Perform a single (sub-)step with a deltatime.
    pub fn step(&mut self, dt: f64) {
        if self.inv_mass == 0.0 {
            return;
        }

        // Position update
        self.prev_pos = self.pos;
        self.vel += dt * self.ext_force / self.inv_mass.recip();
        self.pos += dt * self.vel;

        // Rotation update
        self.prev_rot = self.rot;
        // TODO: do something with the inertia tensor

        // TODO: fix difference between rotations
        self.ang_vel += dt * self.inertia.recip() * self.ext_torque;
        self.rot += dt * self.ang_vel;
    }

    /// Last part of a single (sub-)step.
    pub fn step_finalize(&mut self, damping: f64, dt: f64) {
        self.vel = ((self.pos - self.prev_pos) * damping) / dt;

        // Construct the previous rotation from a sinus and cosinus to invert it
        let (sin, cos) = self.prev_rot.sin_cos();
        // Reconstruct as radians but inverted
        let prev_rot_inv = (-sin).atan2(cos);

        self.ang_vel = (self.rot * prev_rot_inv) / dt;
    }

    /// Apply a force by moving the position, which will trigger velocity increments.
    pub fn apply_force(&mut self, force: Vec2<f64>) {
        self.pos += force;
    }

    /// Apply a rotational force in radians.
    pub fn apply_rotational_force(&mut self, force: f64) {
        self.rot += force;
    }

    /// Set global position.
    pub fn set_position(&mut self, pos: Vec2<f64>, force: bool) {
        self.pos = pos;
        if !force {
            self.prev_pos = pos;
            self.vel = Vec2::zero();
        }
    }

    /// Global position.
    pub fn position(&self) -> Vec2<f64> {
        self.pos
    }

    /// Rotation in radians.
    pub fn rotation(&self) -> f64 {
        self.rot
    }

    /// Inverse of the mass.
    pub fn inverse_mass(&self) -> f64 {
        self.inv_mass
    }

    /// Inertia tensor, corresponds to mass in rotational terms.
    pub fn inertia(&self) -> f64 {
        self.inertia
    }
}

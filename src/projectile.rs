use pixel_game_lib::{
    math::Rotation,
    physics::{rigidbody::RigidBodyHandle, Physics},
};
use vek::Vec2;

use crate::{camera::Camera, object::ObjectSettings, unit::Unit};

/// Spear asset path.
const ASSET_PATH: &str = "projectile.spear-1";
/// Airflow torque strength.
const AIRFLOW_TORQUE: f64 = 30.0;
/// Angular velocity of the projectile must be lower than this.
const AIRFLOW_ANG_VEL_CUTOFF: f64 = 1.0;
/// Projectile velocity must be over this treshold before airflow is applied.
const AIRFLOW_VEL_TRESHOLD: f64 = 50.0;
/// Only apply the force when the offset of the rotation is this close.
const AIRFLOW_ROT_RANGE: f64 = 0.5;

/// Projectile that can fly.
pub struct Projectile {
    /// Reference to the physics rigid body.
    pub rigidbody: RigidBodyHandle,
}

impl Projectile {
    /// Create a new projectile.
    pub fn new(pos: Vec2<f64>, vel: Vec2<f64>, physics: &mut Physics) -> Self {
        puffin::profile_function!();

        // Load the object definition for properties of the object
        let object = crate::asset::<ObjectSettings>(ASSET_PATH);

        let rigidbody = object
            .rigidbody_builder(pos)
            .with_velocity(vel)
            // Set the rotation towards the direction so the torque won't need to adjust too much
            .with_orientation_from_direction(vel.try_normalized().unwrap_or(Vec2::unit_y()))
            .spawn(physics);

        Self { rigidbody }
    }

    /// Update the physics of the projectile.
    ///
    /// Returns whether it should stay alive.
    pub fn update(&self, physics: &mut Physics, units: &mut [Unit], dt: f64) -> bool {
        puffin::profile_scope!("Projectile update");

        let velocity = self.rigidbody.velocity(physics).magnitude();
        if velocity >= AIRFLOW_VEL_TRESHOLD {
            // Let the projectile rotate toward the projectile, simulating air flow
            let dir = Rotation::from_direction(self.rigidbody.velocity(physics).normalized());
            let delta_angle = (dir - self.rigidbody.orientation(physics)).to_radians();

            // Only apply when the angular velocity isn't too much already
            if delta_angle.abs() < AIRFLOW_ROT_RANGE
                && self.rigidbody.angular_velocity(physics).abs() < AIRFLOW_ANG_VEL_CUTOFF
            {
                // The furture away from the required angle the less of an effect we want
                self.rigidbody
                    .apply_torque(delta_angle * AIRFLOW_TORQUE * dt, physics);
            }
        }

        let mut collided = false;
        {
            puffin::profile_scope!("Projectile collision detection");

            // Detect and handle collisions with units
            for collision_key in self.rigidbody.collision_keys_iter(physics) {
                if let Some(unit) = units
                    .iter_mut()
                    .find(move |unit| unit.rigidbody == collision_key)
                {
                    collided = true;
                    unit.health -= 50.0;
                }
            }
        }

        // Destroy when collided, sleeping or out of range
        !collided
            && !self.rigidbody.is_sleeping(physics)
            && physics.is_rigidbody_on_grid(&self.rigidbody)
    }

    /// Render the projectile.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera, physics: &Physics) {
        puffin::profile_function!();

        crate::rotatable_sprite(ASSET_PATH).render(self.rigidbody.iso(physics), canvas, camera);
    }
}

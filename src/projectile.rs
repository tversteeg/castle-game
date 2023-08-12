use vek::Vec2;

use crate::{
    camera::Camera, game::PhysicsEngine, math::Rotation, object::ObjectSettings,
    physics::RigidBodyHandle,
};

/// Spear asset path.
const ASSET_PATH: &str = "projectile.spear-1";
/// Airflow torque strength.
const AIRFLOW_TORQUE: f32 = 30.0;
/// Angular velocity of the projectile must be lower than this.
const AIRFLOW_ANG_VEL_CUTOFF: f32 = 1.0;
/// Projectile velocity must be over this treshold before airflow is applied.
const AIRFLOW_VEL_TRESHOLD: f32 = 50.0;
/// Only apply the force when the offset of the rotation is this close.
const AIRFLOW_ROT_RANGE: f32 = 0.5;

/// Projectile that can fly.
pub struct Projectile {
    /// Reference to the physics rigid body.
    pub rigidbody: RigidBodyHandle,
}

impl Projectile {
    /// Create a new projectile.
    pub fn new(pos: Vec2<f32>, vel: Vec2<f32>, physics: &mut PhysicsEngine) -> Self {
        puffin::profile_function!();

        // Load the object definition for properties of the object
        let object = crate::asset::<ObjectSettings>(ASSET_PATH);

        let rigidbody = physics.add_rigidbody(
            object
                .rigidbody(pos)
                .with_velocity(vel)
                // Set the rotation towards the direction so the torque won't need to adjust too much
                .with_rotation(Rotation::from_direction(
                    vel.try_normalized().unwrap_or(Vec2::unit_y()),
                )),
        );

        Self { rigidbody }
    }

    /// Update the physics of the projectile.
    ///
    /// Returns whether it should stay alive.
    pub fn update(&self, physics: &mut PhysicsEngine, dt: f32) -> bool {
        puffin::profile_function!();

        let rigidbody = self.rigidbody.rigidbody_mut(physics);

        let velocity = rigidbody.velocity().magnitude();
        if velocity >= AIRFLOW_VEL_TRESHOLD {
            // Let the projectile rotate toward the projectile, simulating air flow
            let dir = Rotation::from_direction(rigidbody.direction());
            let delta_angle = (dir - rigidbody.rotation()).to_radians();

            // Only apply when the angular velocity isn't too much already
            if delta_angle.abs() < AIRFLOW_ROT_RANGE
                && rigidbody.angular_velocity().abs() < AIRFLOW_ANG_VEL_CUTOFF
            {
                // The furture away from the required angle the less of an effect we want
                rigidbody.apply_torque(delta_angle * AIRFLOW_TORQUE * dt);
            }
        }

        // Destroy when sleeping
        !rigidbody.is_sleeping()
    }

    /// Render the projectile.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera, physics: &PhysicsEngine) {
        puffin::profile_function!();

        let rigidbody = self.rigidbody.rigidbody(physics);

        crate::rotatable_sprite(ASSET_PATH).render(rigidbody.iso(), canvas, camera);
    }
}

use vek::Vec2;

use crate::{
    camera::Camera,
    game::PhysicsEngine,
    math::Rotation,
    object::ObjectSettings,
    physics::{Physics, RigidBodyHandle},
    terrain::Terrain,
};

/// Spear asset path.
const ASSET_PATH: &str = "projectile.spear-1";
/// Airflow torque strength.
const AIRFLOW_TORQUE: f32 = 20.0;
/// Projectile velocity must be over this treshold before airflow is applied.
const AIRFLOW_VEL_TRESHOLD: f32 = 50.0;
/// Projectile angular velocity must be under this treshold before airflow is applied.
const AIRFLOW_ANG_VEL_TRESHOLD: f32 = 1.0;

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

        let rigidbody = physics.add_rigidbody(object.rigidbody(pos).with_velocity(vel));

        Self { rigidbody }
    }

    /// Update the physics of the projectile.
    ///
    /// Returns whether it should stay alive.
    pub fn update(&self, physics: &mut PhysicsEngine, dt: f32) -> bool {
        puffin::profile_function!();

        let rigidbody = self.rigidbody.rigidbody_mut(physics);

        if rigidbody.velocity().magnitude() >= AIRFLOW_VEL_TRESHOLD
            && rigidbody.angular_velocity() < AIRFLOW_ANG_VEL_TRESHOLD
        {
            // Let the projectile rotate toward the projectile, simulating air flow
            let dir = Rotation::from_direction(rigidbody.direction());
            let delta_angle = rigidbody.rotation() - dir;

            rigidbody.apply_torque(-delta_angle.to_radians() * AIRFLOW_TORQUE * dt);
        }

        !rigidbody.is_sleeping()
    }

    /// Render the projectile.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera, physics: &PhysicsEngine) {
        puffin::profile_function!();

        let rigidbody = self.rigidbody.rigidbody(physics);

        crate::rotatable_sprite(ASSET_PATH).render(rigidbody.iso(), canvas, camera);
    }
}

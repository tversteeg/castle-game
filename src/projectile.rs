use vek::Vec2;

use crate::{
    camera::Camera,
    game::PhysicsEngine,
    object::ObjectSettings,
    physics::{rigidbody::RigidBodyIndex, Physics},
    terrain::Terrain,
};

/// Spear asset path.
const ASSET_PATH: &str = "projectile.spear-1";

/// Projectile that can fly.
pub struct Projectile {
    /// Reference to the physics rigid body.
    pub rigidbody: RigidBodyIndex,
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

    /// Render the projectile.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera, physics: &PhysicsEngine) {
        puffin::profile_function!();

        let rigidbody = physics.rigidbody(self.rigidbody);

        crate::rotatable_sprite(ASSET_PATH).render(rigidbody.iso(), canvas, camera);
    }
}

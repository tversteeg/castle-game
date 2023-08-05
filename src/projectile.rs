use vek::Vec2;

use crate::{
    camera::Camera,
    object::ObjectSettings,
    physics::{rigidbody::RigidBodyIndex, Physics},
    terrain::Terrain,
};

/// Spear asset path.
const ASSET_PATH: &str = "projectile.spear-1";

/// Projectile that can fly.
pub struct Projectile {
    /// Reference to the physics rigid body.
    rigidbody: RigidBodyIndex,
}

impl Projectile {
    /// Create a new projectile.
    pub fn new<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        pos: Vec2<f32>,
        vel: Vec2<f32>,
        physics: &mut Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) -> Self {
        puffin::profile_function!();

        // Load the object definition for properties of the object
        let object = crate::asset::<ObjectSettings>(ASSET_PATH);

        let rigidbody = physics.add_rigidbody(object.rigidbody(pos).with_velocity(vel));

        Self { rigidbody }
    }

    /// Render the projectile.
    pub fn render<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const SIZE: usize,
    >(
        &self,
        canvas: &mut [u32],
        camera: &Camera,
        physics: &Physics<WIDTH, HEIGHT, STEP, BUCKET, SIZE>,
    ) {
        puffin::profile_function!();

        let rigidbody = physics.rigidbody(self.rigidbody);

        crate::rotatable_sprite(ASSET_PATH).render(
            rigidbody.rotation(),
            canvas,
            camera,
            rigidbody.position().as_(),
        );
    }
}

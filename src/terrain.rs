use vek::Vec2;

use crate::{
    camera::Camera,
    game::PhysicsEngine,
    object::ObjectSettings,
    physics::{rigidbody::RigidBody, RigidBodyHandle},
    SIZE,
};

/// Level asset path.
pub const ASSET_PATH: &str = "level.grass-1";

/// Destructible terrain buffer.
pub struct Terrain {
    /// Y offset of the sprite.
    pub y: f32,
    /// Total size of the level.
    pub width: f32,
    /// Physics object reference.
    pub rigidbody: RigidBodyHandle,
}

impl Terrain {
    /// Load a terrain from image bytes.
    pub fn new(physics: &mut PhysicsEngine) -> Self {
        let object = crate::asset::<ObjectSettings>(ASSET_PATH);
        let sprite = crate::sprite(ASSET_PATH);
        let shape = object.shape();

        let width = sprite.width() as f32;
        let y = SIZE.h as f32 - sprite.height() as f32;

        // Create a heightmap for the terrain
        let rigidbody =
            physics.add_rigidbody(RigidBody::new_fixed(Vec2::new(width / 2.0, y), shape));

        Self {
            rigidbody,
            y,
            width,
        }
    }

    /// Draw the terrain based on a camera offset.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        puffin::profile_function!();

        crate::sprite(ASSET_PATH).render(canvas, camera, Vec2::new(0.0, self.y));
    }
}

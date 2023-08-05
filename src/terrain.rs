use vek::{Extent2, Vec2};

use crate::{
    camera::Camera,
    object::ObjectSettings,
    physics::{
        collision::shape::Rectangle,
        rigidbody::{RigidBody, RigidBodyIndex},
        Physics,
    },
    sprite::Sprite,
    SIZE,
};

/// Level asset path.
const ASSET_PATH: &str = "level.grass-1";

/// Destructible terrain buffer.
pub struct Terrain {
    /// Y offset of the sprite.
    pub y: f32,
    /// Total size of the level.
    pub width: f32,
    /// Physics object reference.
    rigidbody: RigidBodyIndex,
}

impl Terrain {
    /// Load a terrain from image bytes.
    pub fn new<
        const WIDTH: u16,
        const HEIGHT: u16,
        const STEP: u16,
        const BUCKET: usize,
        const GRID_SIZE: usize,
    >(
        physics: &mut Physics<WIDTH, HEIGHT, STEP, BUCKET, GRID_SIZE>,
    ) -> Self {
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

        crate::sprite(ASSET_PATH).render(canvas, camera, Vec2::new(0, self.y as i32));
    }
}

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
    pub y: f64,
    /// Total size of the level.
    pub width: f64,
    /// Physics object reference.
    pub rigidbody: RigidBodyHandle,
    /// Array of the top collision point heights of the terrain for ecah pixel.
    top_heights: Vec<u16>,
}

impl Terrain {
    /// Load a terrain from image bytes.
    pub fn new(physics: &mut PhysicsEngine) -> Self {
        let object = crate::asset::<ObjectSettings>(ASSET_PATH);
        let sprite = crate::sprite(ASSET_PATH);
        let shape = object.shape();

        let width = sprite.width() as f64;
        let y = SIZE.h as f64 - sprite.height() as f64;

        // Create a heightmap for the terrain
        let rigidbody =
            physics.add_rigidbody(RigidBody::new_fixed(Vec2::new(width / 2.0, y), shape));

        let top_heights = sprite.top_heights();

        Self {
            rigidbody,
            y,
            width,
            top_heights,
        }
    }

    /// Draw the terrain based on a camera offset.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        puffin::profile_function!();

        crate::sprite(ASSET_PATH).render(canvas, camera, Vec2::new(0.0, self.y));
    }

    /// Whether a point collides with the terrain.
    ///
    /// This doesn't use the collision shape but the actual pixels of the image.
    pub fn point_collides(&self, point: Vec2<f64>, physics: &PhysicsEngine) -> bool {
        // Convert the position to a coordinate that can be used as an index
        let offset = point - self.rigidbody.rigidbody(physics).position() + (self.width / 2.0, 0.0);

        if offset.y < 0.0 || offset.x < 0.0 || offset.x >= self.width {
            false
        } else {
            // Collides if the top height is smaller than the position
            (self.top_heights[offset.x as usize] as f64) < offset.y
        }
    }
}

use serde::Deserialize;
use vek::Vec2;

use crate::{
    camera::Camera,
    graphics::Color,
    object::ObjectSettings,
    physics::{
        rigidbody::{RigidBodyBuilder, RigidBodyHandle},
        Physics,
    },
    solid_shape::SolidShape,
    sprite::{Sprite, SpriteOffset},
    SIZE,
};

/// Level asset path.
pub const ASSET_PATH: &str = "level.grass-1";

/// Destructible procedurally generated terrain buffer.
pub struct Terrain {
    /// Y offset of the sprite.
    pub y: f64,
    /// Physics object reference.
    pub rigidbody: RigidBodyHandle,
    /// Array of the top collision point heights of the terrain for ecah pixel.
    top_heights: Vec<f64>,
    /// Solid shape and sprite.
    shape: SolidShape,
}

impl Terrain {
    /// Load a terrain from image bytes.
    pub fn new(physics: &mut Physics) -> Self {
        let settings = &crate::settings().terrain;

        // Generate random heights
        let mut last = 0.0;
        let mut dir = 0.0;
        let top_heights = (0..crate::settings().terrain.width)
            .map(|i| {
                // Add some noise for every step
                last += (fastrand::f64() - 0.5) * settings.pixel_random_factor * 2.0 + dir;

                // Once every set steps change the direction
                if i % settings.direction_pixels == 0 {
                    dir = (fastrand::f64() - 0.5) * settings.direction_random_factor * 2.0;
                }

                last
            })
            .collect::<Vec<_>>();

        let mut shape = SolidShape::from_heights(
            &top_heights,
            100.0,
            SpriteOffset::LeftTop,
            Color::LightGreen,
            Color::Green,
        );
        shape.generate_sprite();

        let y = SIZE.h as f64 - shape.sprite().height() as f64;

        // Create a heightmap for the terrain
        let rigidbody = {
            let object = crate::asset::<ObjectSettings>(ASSET_PATH);
            let shape = object.shape();
            RigidBodyBuilder::new_static(Vec2::new(settings.width as f64 / 2.0, y))
                .with_collider(shape)
                .spawn(physics)
        };

        Self {
            rigidbody,
            y,
            top_heights,
            shape,
        }
    }

    /// Draw the terrain based on a camera offset.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        puffin::profile_scope!("Render terrain");

        self.shape
            .sprite()
            .render(canvas, camera, Vec2::new(0.0, self.y));
    }

    /// Whether a point collides with the terrain.
    ///
    /// This doesn't use the collision shape but the actual pixels of the image.
    pub fn point_collides(&self, point: Vec2<f64>, physics: &Physics) -> bool {
        // Convert the position to a coordinate that can be used as an index
        let offset = point - (0.0, self.y);
        self.shape.collides(offset)
    }
}

/// Level settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    /// Total length of the terrain.
    pub width: u32,
    /// Random scaling added to each pixel.
    pub pixel_random_factor: f64,
    /// Random scaling added to each direction step of the amount of pixels.
    pub direction_random_factor: f64,
    /// How many pixels before the direction changes.
    pub direction_pixels: u32,
}

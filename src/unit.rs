use blit::prelude::Size;
use vek::{Extent2, Vec2};

use crate::{camera::Camera, sprite::Sprite, terrain::Terrain};

/// How fast the unit walks.
const WALK_SPEED: f64 = 0.2;

/// Unit that can walk on the terrain.
pub struct Unit {
    /// Absolute position.
    pos: Vec2<f64>,
    /// Sprite to render.
    sprite: Sprite,
}

impl Unit {
    /// Load a unit from image bytes.
    pub fn from_bytes(sprite_bytes: &[u8], pos: Vec2<f64>) -> Self {
        let sprite = Sprite::from_bytes(sprite_bytes);

        Self { sprite, pos }
    }

    /// Move the unit.
    pub fn update(&mut self, terrain: &Terrain) {
        if !terrain.point_collides(
            (self.pos + self.ground_collision_point())
                .numcast()
                .unwrap_or_default(),
        ) {
            // No collision with the terrain, the unit falls down
            self.pos.y += 1.0;
        } else if terrain.point_collides(
            (self.pos + self.ground_collision_point() - (0.0, 1.0))
                .numcast()
                .unwrap_or_default(),
        ) {
            // The unit has sunk into the terrain, move it up
            self.pos.y -= 1.0;
        } else {
            // Collision with the terrain, the unit walks to the right
            self.pos.x += WALK_SPEED;
        }
    }

    /// Draw the unit.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        self.sprite
            .render(canvas, camera, self.pos.numcast().unwrap_or_default());
    }

    /// Where the unit collides with the ground.
    fn ground_collision_point(&self) -> Vec2<f64> {
        (
            self.sprite.width() as f64 / 2.0,
            self.sprite.height() as f64 - 2.0,
        )
            .into()
    }
}

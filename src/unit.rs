use vek::Vec2;

use crate::{assets::Assets, camera::Camera, sprite::Sprite, terrain::Terrain};

/// How fast the unit walks.
const WALK_SPEED: f64 = 0.2;
/// Unit base asset path.
const BASE_ASSET_PATH: &str = "unit.base-1";
/// Unit hands with spear asset path.
const SPEAR_HANDS_ASSET_PATH: &str = "unit.spear-hands-1";

/// Unit that can walk on the terrain.
pub struct Unit {
    /// Absolute position.
    pos: Vec2<f64>,
    /// Assets reference for rendering the sprites.
    assets: &'static Assets,
}

impl Unit {
    /// Create a new unit.
    pub fn new(pos: Vec2<f64>, assets: &'static Assets) -> Self {
        Self { assets, pos }
    }

    /// Move the unit.
    pub fn update(&mut self, terrain: &Terrain) {
        if !terrain.point_collides(self.pos.numcast().unwrap_or_default()) {
            // No collision with the terrain, the unit falls down
            self.pos.y += 1.0;
        } else if terrain.point_collides((self.pos - (0.0, 1.0)).numcast().unwrap_or_default()) {
            // The unit has sunk into the terrain, move it up
            self.pos.y -= 1.0;
        } else {
            // Collision with the terrain, the unit walks to the right
            self.pos.x += WALK_SPEED;
        }
    }

    /// Draw the unit.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        self.assets.sprite(BASE_ASSET_PATH).render(
            canvas,
            camera,
            (self.pos - self.ground_collision_point())
                .numcast()
                .unwrap_or_default(),
        );

        self.assets.sprite(SPEAR_HANDS_ASSET_PATH).render(
            canvas,
            camera,
            (self.pos - (1.0, 1.0) - self.ground_collision_point())
                .numcast()
                .unwrap_or_default(),
        );
    }

    /// Where the unit collides with the ground.
    fn ground_collision_point(&self) -> Vec2<f64> {
        let sprite = self.assets.sprite(BASE_ASSET_PATH);

        (sprite.width() as f64 / 2.0, sprite.height() as f64 - 2.0).into()
    }
}

use vek::Vec2;

use crate::{camera::Camera, terrain::Terrain};

/// Spear asset path.
const ASSET_PATH: &str = "projectile.spear-1";

/// Projectile that can fly.
pub struct Projectile {
    /// Absolute position.
    pos: Vec2<f32>,
    /// Velocity.
    vel: Vec2<f32>,
}

impl Projectile {
    /// Create a new unit.
    pub fn new(pos: Vec2<f32>, vel: Vec2<f32>) -> Self {
        Self { pos, vel }
    }

    /// Move the projectile.
    ///
    /// Returns whether the projectile should be removed.
    pub fn update(&mut self, terrain: &Terrain, dt: f32) -> bool {
        puffin::profile_function!();

        self.pos += self.vel * dt;
        self.vel.y += crate::settings().projectile_gravity;

        terrain.point_collides(self.pos.numcast().unwrap_or_default())
    }

    /// Render the projectile.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        puffin::profile_function!();

        let rotation = self.vel.y.atan2(self.vel.x);

        crate::rotatable_sprite(ASSET_PATH).render(
            rotation,
            canvas,
            camera,
            self.pos.numcast().unwrap_or_default(),
        );
    }
}

use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::Vec2;

use crate::{
    assets::Assets, camera::Camera, projectile::Projectile, terrain::Terrain, timer::Timer,
};

/// Unit settings asset path.
const SETTINGS_ASSET_PATH: &str = "unit.spear";
/// Unit base asset path.
const BASE_ASSET_PATH: &str = "unit.base-1";
/// Unit hands with spear asset path.
const SPEAR_HANDS_ASSET_PATH: &str = "unit.spear-hands-1";

/// Unit that can walk on the terrain.
pub struct Unit {
    /// Absolute position.
    pos: Vec2<f64>,
    /// Timer for throwing a spear.
    projectile_timer: Timer,
    /// How long to hide the hands after a spear is thrown.
    hide_hands_delay: f64,
}

impl Unit {
    /// Create a new unit.
    pub fn new(pos: Vec2<f64>, assets: &'static Assets) -> Self {
        let projectile_timer = Timer::new(
            assets
                .asset::<Settings>(SETTINGS_ASSET_PATH)
                .projectile_spawn_interval,
        );

        let hide_hands_delay = 0.0;

        Self {
            pos,
            projectile_timer,
            hide_hands_delay,
        }
    }

    /// Move the unit.
    ///
    /// When a projectile is returned one is spawned.
    pub fn update(
        &mut self,
        terrain: &Terrain,
        dt: f64,
        assets: &'static Assets,
    ) -> Option<Projectile> {
        if !terrain.point_collides(self.pos.numcast().unwrap_or_default()) {
            // No collision with the terrain, the unit falls down
            self.pos.y += 1.0;
        } else if terrain.point_collides((self.pos - (0.0, 1.0)).numcast().unwrap_or_default()) {
            // The unit has sunk into the terrain, move it up
            self.pos.y -= 1.0;
        } else {
            // Collision with the terrain, the unit walks to the right
            self.pos.x += assets.asset::<Settings>(SETTINGS_ASSET_PATH).walk_speed * dt;
        }

        // Update hands delay
        if self.hide_hands_delay > 0.0 {
            self.hide_hands_delay -= dt;
        }

        // Spawn a projectile if timer runs out
        if self.projectile_timer.update(dt) {
            let settings = assets.asset::<Settings>(SETTINGS_ASSET_PATH);

            let velocity = settings.projectile_velocity;
            self.hide_hands_delay = settings.hide_hands_delay;

            Some(Projectile::new(self.pos, Vec2::new(velocity, -velocity)))
        } else {
            None
        }
    }

    /// Draw the unit.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera, assets: &'static Assets) {
        assets.sprite(BASE_ASSET_PATH).render(
            canvas,
            camera,
            (self.pos - self.ground_collision_point(assets))
                .numcast()
                .unwrap_or_default(),
        );

        if self.hide_hands_delay <= 0.0 {
            assets.sprite(SPEAR_HANDS_ASSET_PATH).render(
                canvas,
                camera,
                (self.pos - (1.0, 1.0) - self.ground_collision_point(assets))
                    .numcast()
                    .unwrap_or_default(),
            );
        }
    }

    /// Where the unit collides with the ground.
    fn ground_collision_point(&self, assets: &'static Assets) -> Vec2<f64> {
        let sprite = assets.sprite(BASE_ASSET_PATH);

        (sprite.width() as f64 / 2.0, sprite.height() as f64 - 2.0).into()
    }
}

/// Unit settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    /// How many pixels a unit moves in a second.
    pub walk_speed: f64,
    /// Interval in seconds for when a new projectile is thrown.
    pub projectile_spawn_interval: f64,
    /// How fast a projectile is thrown.
    pub projectile_velocity: f64,
    /// How long the hands are hidden after launching a projectile.
    pub hide_hands_delay: f64,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

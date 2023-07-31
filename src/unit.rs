use assets_manager::{loader::TomlLoader, Asset, AssetGuard};
use serde::Deserialize;
use vek::Vec2;

use crate::{
    camera::Camera, projectile::Projectile, random::RandomRangeF32, terrain::Terrain, timer::Timer,
};

/// All unit types.
#[derive(Debug, Clone, Copy)]
pub enum UnitType {
    PlayerSpear,
    EnemySpear,
}

impl UnitType {
    /// Settings path to load for this type.
    pub fn settings(&self) -> AssetGuard<Settings> {
        // Settings asset path
        let path = match self {
            Self::PlayerSpear => "unit.spear",
            Self::EnemySpear => "unit.enemy-spear",
        };

        crate::asset(path)
    }
}

/// Unit that can walk on the terrain.
#[derive(Debug)]
pub struct Unit {
    /// Type of the unit, used to find the settings.
    r#type: UnitType,
    /// Absolute position.
    pos: Vec2<f32>,
    /// Timer for throwing a spear.
    projectile_timer: Timer,
    /// How long to hide the hands after a spear is thrown.
    hide_hands_delay: f32,
}

impl Unit {
    /// Create a new unit.
    pub fn new(pos: Vec2<f32>, r#type: UnitType) -> Self {
        let projectile_timer = Timer::new(r#type.settings().projectile_spawn_interval);

        let hide_hands_delay = 0.0;

        Self {
            r#type,
            pos,
            projectile_timer,
            hide_hands_delay,
        }
    }

    /// Move the unit.
    ///
    /// When a projectile is returned one is spawned.
    pub fn update(&mut self, terrain: &Terrain, dt: f32) -> Option<Projectile> {
        puffin::profile_function!();

        if !terrain.point_collides(self.pos.numcast().unwrap_or_default()) {
            // No collision with the terrain, the unit falls down
            self.pos.y += 1.0;
        } else if terrain.point_collides((self.pos - (0.0, 1.0)).numcast().unwrap_or_default()) {
            // The unit has sunk into the terrain, move it up
            self.pos.y -= 1.0;
        } else {
            // Collision with the terrain, the unit walks to the right
            let walk_speed = self.settings().walk_speed;
            self.pos.x += walk_speed * dt;
        }

        // Update hands delay
        if self.hide_hands_delay > 0.0 {
            self.hide_hands_delay -= dt;
        }

        // Spawn a projectile if timer runs out
        if self.projectile_timer.update(dt) {
            let hide_hands_delay = self.settings().hide_hands_delay;
            self.hide_hands_delay = hide_hands_delay;

            let velocity = self.settings().projectile_velocity.value();

            Some(Projectile::new(self.pos, Vec2::new(velocity, -velocity)))
        } else {
            None
        }
    }

    /// Draw the unit.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera) {
        puffin::profile_function!();

        let settings = self.settings();

        crate::sprite(&settings.base_asset_path).render(
            canvas,
            camera,
            (self.pos - self.ground_collision_point())
                .numcast()
                .unwrap_or_default(),
        );

        if let Some(hands_asset_path) = &settings.hands_asset_path {
            if self.hide_hands_delay <= 0.0 {
                crate::sprite(hands_asset_path).render(
                    canvas,
                    camera,
                    (self.pos - (1.0, 1.0) - self.ground_collision_point())
                        .numcast()
                        .unwrap_or_default(),
                );
            }
        }
    }

    /// Where the unit collides with the ground.
    fn ground_collision_point(&self) -> Vec2<f32> {
        let base_asset_path = &self.settings().base_asset_path;
        let sprite = crate::sprite(base_asset_path);

        (sprite.width() as f32 / 2.0, sprite.height() as f32 - 2.0).into()
    }

    /// The settings for this unit.
    fn settings(&self) -> AssetGuard<Settings> {
        self.r#type.settings()
    }
}

/// Unit settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    /// Asset path for the base.
    ///
    /// Usually this is the body.
    pub base_asset_path: String,
    /// Asset path for the hands.
    pub hands_asset_path: Option<String>,
    /// Asset path for the projectile.
    pub projectile_asset_path: Option<String>,
    /// Who the unit belongs to.
    pub allegiance: Allegiance,
    /// How many pixels a unit moves in a second.
    pub walk_speed: f32,
    /// Interval in seconds for when a new projectile is thrown.
    pub projectile_spawn_interval: f32,
    /// How fast a projectile is thrown.
    pub projectile_velocity: RandomRangeF32,
    /// How long the hands are hidden after launching a projectile.
    pub hide_hands_delay: f32,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

/// Player unit or enemy unit.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Allegiance {
    /// Unit belongs to the player.
    Player,
    /// Unit is controlled by enemy AI.
    Enemy,
}

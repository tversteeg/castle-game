use assets_manager::{loader::TomlLoader, Asset, AssetGuard};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{
    camera::Camera,
    object::ObjectSettings,
    physics::{
        rigidbody::{RigidBodyHandle},
        Physics,
    },
    projectile::Projectile,
    random::RandomRangeF64,
    terrain::Terrain,
    timer::Timer,
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

    /// Asset path based on what type to load.
    pub fn asset_path(&self) -> &'static str {
        match self {
            Self::PlayerSpear => "unit.spear",
            Self::EnemySpear => "unit.enemy-spear",
        }
    }
}

/// Unit that can walk on the terrain.
#[derive(Debug)]
pub struct Unit {
    /// Type of the unit, used to find the settings.
    r#type: UnitType,
    /// Absolute position.
    pos: Vec2<f64>,
    /// Timer for throwing a spear.
    projectile_timer: Timer,
    /// How long to hide the hands after a spear is thrown.
    hide_hands_delay: f64,
    /// How much health the unit has currently.
    pub health: f64,
    /// Collision shape.
    pub rigidbody: RigidBodyHandle,
}

impl Unit {
    /// Create a new unit.
    pub fn new(pos: Vec2<f64>, r#type: UnitType, physics: &mut Physics) -> Self {
        let projectile_timer = Timer::new(r#type.settings().projectile_spawn_interval);

        let hide_hands_delay = 0.0;
        let health = r#type.settings().health;

        // Load the object definition for properties of the object
        let object = crate::asset::<ObjectSettings>(r#type.asset_path());
        let rigidbody = object.rigidbody_builder(pos).spawn(physics);

        Self {
            r#type,
            pos,
            projectile_timer,
            hide_hands_delay,
            health,
            rigidbody,
        }
    }

    /// Move the unit.
    ///
    /// When a projectile is returned one is spawned.
    pub fn update(
        &mut self,
        terrain: &Terrain,
        dt: f64,
        physics: &mut Physics,
    ) -> Option<Projectile> {
        puffin::profile_scope!("Unit update");

        // Update rigidbody position
        self.rigidbody.set_position(self.pos, physics);

        if !terrain.point_collides(self.pos, physics) {
            // No collision with the terrain, the unit falls down
            self.pos.y += 1.0;
        } else if terrain.point_collides(self.pos - (0.0, 1.0), physics) {
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

            Some(Projectile::new(
                self.pos + self.settings().projectile_spawn_offset,
                Vec2::new(velocity, -velocity),
                physics,
            ))
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

        // Draw the healthbar
        crate::graphics::healthbar::healthbar(
            self.health,
            settings.health,
            self.pos + settings.healthbar_offset,
            settings.healthbar_size,
            canvas,
            camera,
        );
    }

    /// Where the unit collides with the ground.
    fn ground_collision_point(&self) -> Vec2<f64> {
        let base_asset_path = &self.settings().base_asset_path;
        let sprite = crate::sprite(base_asset_path);

        (sprite.width() as f64 / 2.0, sprite.height() as f64 - 2.0).into()
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
    pub walk_speed: f64,
    /// How much health the unit has on spawn.
    pub health: f64,
    /// Interval in seconds for when a new projectile is thrown.
    pub projectile_spawn_interval: f64,
    /// Offset in pixels from the center of the unit body from where the projectile is thrown.
    pub projectile_spawn_offset: Vec2<f64>,
    /// How fast a projectile is thrown.
    pub projectile_velocity: RandomRangeF64,
    /// How long the hands are hidden after launching a projectile.
    pub hide_hands_delay: f64,
    /// Size of the healthbar.
    pub healthbar_size: Extent2<f32>,
    /// Position offset of the healthbar.
    pub healthbar_offset: Vec2<f64>,
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

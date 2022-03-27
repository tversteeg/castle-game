use crate::{
    inspector::Inspectable,
    random::RandomRange,
    unit::{faction::Faction, unit_type::UnitType},
};
use bevy::math::Vec2;

/// The constants.
///
/// Not made actually constant so it can be changed in the inspector.
#[derive(Debug, Inspectable)]
pub struct Constants {
    /// World constants.
    #[inspectable(label = "World", collapse)]
    pub world: WorldConstants,
    /// Camera constants.
    #[inspectable(label = "Camera", collapse)]
    pub camera: CameraConstants,
    /// Terrain constants.
    #[inspectable(label = "Terrain", collapse)]
    pub terrain: TerrainConstants,
    /// Enemy spawning constants.
    #[inspectable(label = "Spawning", collapse)]
    pub spawning: SpawningConstants,
    /// Ally soldier constants.
    #[inspectable(label = "Ally Soldier", collapse)]
    pub ally_soldier: UnitConstants,
    /// Enemy soldier constants.
    #[inspectable(label = "Enemy Soldier", collapse)]
    pub enemy_soldier: UnitConstants,
    /// Ally archer constants.
    #[inspectable(label = "Ally Archer", collapse)]
    pub ally_archer: UnitConstants,
    /// Enemy archer constants.
    #[inspectable(label = "Enemy Archer", collapse)]
    pub enemy_archer: UnitConstants,
    /// Arrow constants.
    #[inspectable(label = "Arrow", collapse)]
    pub arrow: ProjectileConstants,
    /// UI constants.
    #[inspectable(label = "UI", collapse)]
    pub ui: UiConstants,
}

impl Constants {
    /// Get the unit constants.
    pub fn unit(&'_ self, unit_type: UnitType, faction: Faction) -> &'_ UnitConstants {
        match (unit_type, faction) {
            (UnitType::Soldier, Faction::Ally) => &self.ally_soldier,
            (UnitType::Soldier, Faction::Enemy) => &self.enemy_soldier,
            (UnitType::Archer, Faction::Ally) => &self.ally_archer,
            (UnitType::Archer, Faction::Enemy) => &self.enemy_archer,
        }
    }
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            ally_soldier: UnitConstants {
                hp: 100.0,
                walking_speed: 1.5,
                minimum_weapon_distance: 2.0,
                weapon_delay: 1.0,
                stop_distance: 2.0,
            },
            enemy_soldier: UnitConstants {
                hp: 100.0,
                walking_speed: -1.7,
                minimum_weapon_distance: 2.0,
                weapon_delay: 1.0,
                stop_distance: 2.0,
            },
            ally_archer: UnitConstants {
                hp: 100.0,
                walking_speed: 1.2,
                minimum_weapon_distance: 100.0,
                weapon_delay: 5.0,
                stop_distance: 50.0,
            },
            enemy_archer: UnitConstants {
                hp: 100.0,
                walking_speed: -1.3,
                minimum_weapon_distance: 100.0,
                weapon_delay: 5.0,
                stop_distance: 50.0,
            },
            arrow: ProjectileConstants {
                remove_after_resting_for: 0.5,
                flight_time: 5.0,
                rotation_offset: -std::f32::consts::PI / 2.0,
            },
            spawning: SpawningConstants::default(),
            terrain: TerrainConstants::default(),
            camera: CameraConstants::default(),
            world: WorldConstants::default(),
            ui: UiConstants::default(),
        }
    }
}

/// Constants for a specific unit.
#[derive(Debug, Clone, Inspectable)]
pub struct UnitConstants {
    /// Health.
    #[inspectable(min = 1.0, max = 1000.0)]
    pub hp: f32,
    /// Walking speed.
    #[inspectable(min = -100.0, max = 100.0, suffix = "m/s")]
    pub walking_speed: f32,
    /// The minimum distance at which the weapon will be used.
    #[inspectable(min = 0.0, max = 1000.0, suffix = "m")]
    pub minimum_weapon_distance: f32,
    /// The amount of seconds at which the weapon will be used.
    #[inspectable(min = 0.2, max = 1000.0, suffix = "s")]
    pub weapon_delay: f32,
    /// Distance at which to stop before the next unit.
    #[inspectable(min = 0.2, max = 1000.0, suffix = "m")]
    pub stop_distance: f32,
}

/// Constants for the terrain.
#[derive(Debug, Clone, Inspectable)]
pub struct TerrainConstants {
    /// Total width of the terrain.
    #[inspectable(min = 10.0, max = 1000.0, suffix = "m")]
    pub width: f32,
    /// How many height points should be calculated for the terrain.
    pub height_points: usize,
    /// Random height of the terrain between the bounds.
    #[inspectable(min = 1.0, max = 50.0, suffix = "m")]
    pub height: RandomRange,
    /// The scale of the noise, will influence which X points will be get as sample.
    #[inspectable(min = 0.0, max = 1.0)]
    pub noise_scale: f64,
    /// Where allies are spawned.
    #[inspectable(min = 0.0, max = 1000.0, suffix = "m")]
    pub ally_starting_position: f32,
    /// Where enemies are spawned.
    #[inspectable(min = 0.0, max = 1000.0, suffix = "m")]
    pub enemy_starting_position: f32,
}

impl Default for TerrainConstants {
    fn default() -> Self {
        let width = 300.0;

        Self {
            width,
            height_points: 100,
            height: RandomRange {
                min: 6.0,
                max: 14.0,
            },
            noise_scale: 0.01,
            ally_starting_position: 5.0,
            enemy_starting_position: width - 5.0,
        }
    }
}

/// Constants for the camera.
#[derive(Debug, Clone, Copy, Inspectable)]
pub struct CameraConstants {
    /// How far the camera is zoomed in.
    pub scale: f32,
    /// Camera border on the each on which it won't move.
    pub border_size: f32,
}

impl Default for CameraConstants {
    fn default() -> Self {
        Self {
            scale: 1.0 / 10.0,
            border_size: 0.2,
        }
    }
}

/// Constants for projectiles.
#[derive(Debug, Clone, Copy, Inspectable)]
pub struct ProjectileConstants {
    /// How long until an arrow is removed when laying on the ground.
    #[inspectable(min = 0.0, max = 1000.0, suffix = "s")]
    pub remove_after_resting_for: f32,
    /// Seconds until the arrow will hit the target.
    #[inspectable(min = 0.0, max = 1000.0, suffix = "s")]
    pub flight_time: f32,
    /// How much the rotation of the arrow will be offset.
    #[inspectable(min = -std::f32::consts::PI, max = std::f32::consts::PI, suffix = "r")]
    pub rotation_offset: f32,
}

/// Constants for the world.
#[derive(Debug, Clone, Copy, Inspectable)]
pub struct WorldConstants {
    /// How fast objects fall in m/s.
    #[inspectable(min = -1000.0, max = 1000.0, suffix = "m/s")]
    pub gravity: f32,
}

impl Default for WorldConstants {
    fn default() -> Self {
        Self { gravity: -9.81 }
    }
}

/// Constants for the spawning of enemies and allies.
#[derive(Debug, Clone, Copy, Inspectable)]
pub struct SpawningConstants {
    /// When the recruit button for an allied soldier is available again.
    #[inspectable(min = 0.1, max = 1000.0, suffix = "s")]
    pub ally_soldier_interval: f32,
    /// When the recruit button for an allied archer is available again.
    #[inspectable(min = 0.1, max = 1000.0, suffix = "s")]
    pub ally_archer_interval: f32,
    /// How long it takes for an enemy soldier to spawn.
    #[inspectable(min = 0.1, max = 1000.0, suffix = "s")]
    pub enemy_soldier_interval: f32,
    /// How long it takes for an enemy archer to spawn.
    #[inspectable(min = 0.1, max = 1000.0, suffix = "s")]
    pub enemy_archer_interval: f32,
}

impl Default for SpawningConstants {
    fn default() -> Self {
        Self {
            ally_soldier_interval: 5.0,
            ally_archer_interval: 10.0,
            enemy_soldier_interval: 10.0,
            enemy_archer_interval: 21.0,
        }
    }
}

/// Constants for the UI.
#[derive(Debug, Clone, Copy, Inspectable)]
pub struct UiConstants {
    /// How far the main bar is removed from the top half of the screen.
    pub main_bar_offset: Vec2,
    /// The size of the recruit button and the progress bar.
    pub recruit_button_size: Vec2,
}

impl Default for UiConstants {
    fn default() -> Self {
        Self {
            main_bar_offset: [0.0, 5.0].into(),
            recruit_button_size: [80.0, 20.0].into(),
        }
    }
}

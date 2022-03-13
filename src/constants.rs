use crate::{
    inspector::Inspectable,
    unit::{faction::Faction, unit_type::UnitType},
};

/// The constants.
///
/// Not made actually constant so it can be changed in the inspector.
#[derive(Debug, Inspectable)]
pub struct Constants {
    /// Camera constants.
    #[inspectable(label = "Camera", collapse)]
    pub camera: CameraConstants,
    /// Terrain constants.
    #[inspectable(label = "Terrain", collapse)]
    pub terrain: TerrainConstants,
    /// Ally soldier constants.
    #[inspectable(label = "Ally Soldier", collapse)]
    ally_soldier: UnitConstants,
    /// Enemy soldier constants.
    #[inspectable(label = "Enemy Soldier", collapse)]
    enemy_soldier: UnitConstants,
    /// Ally archer constants.
    #[inspectable(label = "Ally Archer", collapse)]
    ally_archer: UnitConstants,
    /// Enemy archer constants.
    #[inspectable(label = "Enemy Archer", collapse)]
    enemy_archer: UnitConstants,
}

impl Constants {
    /// Get the unit constants.
    pub fn unit(&self, unit_type: UnitType, faction: Faction) -> UnitConstants {
        match (unit_type, faction) {
            (UnitType::Soldier, Faction::Ally) => self.ally_soldier,
            (UnitType::Soldier, Faction::Enemy) => self.enemy_soldier,
            (UnitType::Archer, Faction::Ally) => self.ally_archer,
            (UnitType::Archer, Faction::Enemy) => self.enemy_archer,
        }
    }
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            ally_soldier: UnitConstants {
                hp: 100.0,
                walking_speed: 1.5,
            },
            enemy_soldier: UnitConstants {
                hp: 100.0,
                walking_speed: -1.7,
            },
            ally_archer: UnitConstants {
                hp: 100.0,
                walking_speed: 1.2,
            },
            enemy_archer: UnitConstants {
                hp: 100.0,
                walking_speed: -1.3,
            },
            terrain: TerrainConstants::default(),
            camera: CameraConstants::default(),
        }
    }
}

/// Constants for a specific unit.
#[derive(Debug, Clone, Copy, Inspectable)]
pub struct UnitConstants {
    /// Health.
    #[inspectable(min = 1.0, max = 1000.0)]
    pub hp: f32,
    /// Walking speed.
    #[inspectable(min = -100.0, max = 100.0, suffix = "m/s")]
    pub walking_speed: f32,
}

/// Constants for the terrain.
#[derive(Debug, Clone, Inspectable)]
pub struct TerrainConstants {
    /// Total width of the terrain.
    #[inspectable(min = 10.0, max = 1000.0, suffix = "m")]
    pub width: f32,
    /// How many height points should be calculated for the terrain.
    pub height_points: usize,
    /// Minimum height of the terrain.
    #[inspectable(min = 1.0, max = 50.0, suffix = "m")]
    pub min_height: f32,
    /// Maximum height of the terrain.
    #[inspectable(min = 0.0, max = 50.0, suffix = "m")]
    pub max_height: f32,
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
            min_height: 6.0,
            max_height: 14.0,
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

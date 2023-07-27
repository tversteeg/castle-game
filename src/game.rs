use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;

use crate::{
    assets::Assets,
    camera::Camera,
    debug::DebugDraw,
    input::Input,
    physics::Settings as PhysicsSettings,
    projectile::Projectile,
    terrain::Terrain,
    timer::Timer,
    unit::{Unit, UnitType},
    SIZE,
};

/// Handles everything related to the game.
pub struct GameState {
    /// Reference to all assets.
    assets: &'static Assets,
    /// First level ground.
    terrain: Terrain,
    /// Timer for when a unit should spawn.
    unit_spawner: Timer,
    /// Timer for when an enemy unit should spawn.
    enemy_unit_spawner: Timer,
    /// Units on the map.
    units: Vec<Unit>,
    /// Projectiles flying around.
    projectiles: Vec<Projectile>,
    /// Camera position based on the cursor.
    camera: Camera,
    /// Maximum X position of the level.
    level_width: u32,
    /// Debug information on the screen.
    debug_state: DebugDraw,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new(assets: &'static Assets) -> Self {
        let terrain = Terrain::new(assets);
        let units = Vec::new();
        let unit_spawner = Timer::new(assets.settings().unit_spawn_interval);
        let enemy_unit_spawner = Timer::new(assets.settings().enemy_unit_spawn_interval);
        let projectiles = Vec::new();
        let level_width = terrain.width();
        let camera = Camera::default();
        let debug_state = DebugDraw::new();

        Self {
            debug_state,
            projectiles,
            assets,
            terrain,
            units,
            unit_spawner,
            enemy_unit_spawner,
            camera,
            level_width,
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], frame_time: f32) {
        self.assets.font("font.torus-sans").render(
            canvas,
            &format!("Castle Game: {frame_time}"),
            0,
            0,
        );

        self.terrain.render(canvas, &self.camera, self.assets);

        // Render all units
        self.units
            .iter()
            .for_each(|unit| unit.render(canvas, &self.camera, self.assets));

        // Render all projectiles
        self.projectiles
            .iter()
            .for_each(|projectile| projectile.render(canvas, &self.camera, self.assets));

        // Render debug information
        self.debug_state.render(canvas, self.assets);
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input, dt: f32) {
        let settings = self.assets.settings();

        // Move the camera based on the mouse position
        if input.mouse_pos.x <= settings.pan_edge_offset {
            self.camera.pan(
                -settings.pan_speed * dt,
                0.0,
                0.0,
                (self.level_width - SIZE.w as u32) as f32,
            );
        } else if input.mouse_pos.x >= SIZE.w as i32 - settings.pan_edge_offset {
            self.camera.pan(
                settings.pan_speed * dt,
                0.0,
                0.0,
                (self.level_width - SIZE.w as u32) as f32,
            );
        }

        // Update all units
        self.units.iter_mut().for_each(|unit| {
            if let Some(projectile) = unit.update(&self.terrain, dt, self.assets) {
                self.projectiles.push(projectile);
            }
        });

        // Update all projectiles
        self.projectiles
            .retain_mut(|projectile| !projectile.update(&self.terrain, dt, self.assets));

        // Update the spawn timers and spawn a unit when it ticks
        if self.unit_spawner.update(dt) {
            // Spawn a unit at the upper edge of the terrain image
            self.units.push(Unit::new(
                (10.0, self.terrain.y_offset() as f32).into(),
                UnitType::PlayerSpear,
                self.assets,
            ));
        }
        if self.enemy_unit_spawner.update(dt) {
            // Spawn a unit at the upper edge of the terrain image
            self.units.push(Unit::new(
                (
                    self.level_width as f32 - 10.0,
                    self.terrain.y_offset() as f32,
                )
                    .into(),
                UnitType::EnemySpear,
                self.assets,
            ));
        }

        // Update debug information
        self.debug_state.update(input, dt, self.assets);
    }
}

/// Game settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    /// Distance from the edge at which the camera will pan.
    pub pan_edge_offset: i32,
    /// How many pixels per second the camera will pan.
    pub pan_speed: f32,
    /// Interval in seconds for when a unit spawns.
    pub unit_spawn_interval: f32,
    /// Interval in seconds for when an enemy unit spawns.
    pub enemy_unit_spawn_interval: f32,
    /// Downward force on all projectiles.
    pub projectile_gravity: f32,
    /// Physics settings.
    pub physics: PhysicsSettings,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

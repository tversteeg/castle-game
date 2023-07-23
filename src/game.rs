use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;

use crate::{
    assets::Assets, camera::Camera, input::Input, terrain::Terrain, timer::Timer, unit::Unit, SIZE,
};

/// Handles everything related to the game.
pub struct GameState {
    /// Reference to all assets.
    assets: &'static Assets,
    /// First level ground.
    terrain: Terrain,
    /// Timer for when a unit should spawn.
    unit_spawner: Timer,
    /// Units on the map.
    units: Vec<Unit>,
    /// Camera position based on the cursor.
    camera: Camera,
    /// Maximum X position of the level.
    level_width: u32,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new(assets: &'static Assets) -> Self {
        // Load the terrain
        let terrain = Terrain::new(assets);

        // Load the embedded unit
        let units = Vec::new();
        let unit_spawner = Timer::new(assets.settings().unit_spawn_interval);

        let level_width = terrain.width();

        let camera = Camera::default();

        Self {
            assets,
            terrain,
            units,
            unit_spawner,
            camera,
            level_width,
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], frame_time: f64) {
        self.assets.font("font.torus-sans").render(
            canvas,
            &format!("Castle Game: {frame_time}"),
            0,
            0,
        );

        self.terrain.render(canvas, &self.camera);

        // Render all units
        self.units
            .iter()
            .for_each(|unit| unit.render(canvas, &self.camera));
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input, dt: f64) {
        let settings = self.assets.settings();

        // Move the camera based on the mouse position
        if input.mouse_pos.x <= settings.pan_edge_offset {
            self.camera.pan(
                -settings.pan_speed * dt,
                0.0,
                0.0,
                (self.level_width - SIZE.w as u32) as f64,
            );
        } else if input.mouse_pos.x >= SIZE.w as i32 - settings.pan_edge_offset {
            self.camera.pan(
                settings.pan_speed * dt,
                0.0,
                0.0,
                (self.level_width - SIZE.w as u32) as f64,
            );
        }

        // Update all units
        self.units
            .iter_mut()
            .for_each(|unit| unit.update(&self.terrain, dt));

        // Update the spawn timer and spawn a unit when it ticks
        if self.unit_spawner.update(dt) {
            // Spawn a unit at the upper edge of the terrain image
            self.units.push(Unit::new(
                (10.0, self.terrain.y_offset() as f64).into(),
                self.assets,
            ));
        }
    }
}

/// Game settings loaded from a file so it's easier to change them with hot-reloading.
#[derive(Deserialize)]
pub struct Settings {
    /// Distance from the edge at which the camera will pan.
    pub pan_edge_offset: i32,
    /// How many pixels per second the camera will pan.
    pub pan_speed: f64,
    /// Interval in seconds for when a unit spawns.
    pub unit_spawn_interval: f64,
    /// How many pixels a unit moves in a second.
    pub unit_speed: f64,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

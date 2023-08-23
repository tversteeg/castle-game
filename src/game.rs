use assets_manager::{loader::TomlLoader, Asset};
use serde::Deserialize;
use vek::Vec2;

#[cfg(feature = "debug")]
use crate::debug::{DebugDraw, DebugSettings};
use crate::{
    camera::Camera,
    input::Input,
    physics::{Physics, Settings as PhysicsSettings},
    projectile::Projectile,
    terrain::Settings as TerrainSettings,
    terrain::Terrain,
    timer::Timer,
    unit::{Unit, UnitType},
    SIZE,
};

/// Handles everything related to the game.
pub struct GameState {
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
    /// Physics engine.
    ///
    /// Size of the grid is the maximum size of any map.
    physics: Physics,
    /// Debug information on the screen.
    #[cfg(feature = "debug")]
    debug_state: DebugDraw,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new() -> Self {
        let units = Vec::new();
        let mut unit_spawner = Timer::new(crate::settings().unit_spawn_interval);
        unit_spawner.trigger();
        let enemy_unit_spawner = Timer::new(crate::settings().enemy_unit_spawn_interval);
        let projectiles = Vec::new();
        let camera = Camera::default();
        let mut physics = Physics::new();
        let terrain = Terrain::new(&mut physics);

        Self {
            projectiles,
            terrain,
            units,
            unit_spawner,
            enemy_unit_spawner,
            camera,
            physics,
            #[cfg(feature = "debug")]
            debug_state: DebugDraw::new(),
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], _frame_time: f64) {
        self.terrain.render(canvas, &self.camera);

        // Render all units
        self.units
            .iter()
            .for_each(|unit| unit.render(canvas, &self.camera));

        // Render all projectiles
        self.projectiles
            .iter()
            .for_each(|projectile| projectile.render(canvas, &self.camera, &self.physics));

        // Render debug information
        #[cfg(feature = "debug")]
        self.debug_state
            .render(&mut self.physics, &self.camera, canvas);
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input, dt: f64) {
        let settings = crate::settings();

        // Move the camera based on the mouse position
        if input.mouse_pos.x <= settings.pan_edge_offset {
            self.camera.pan(
                -settings.pan_speed * dt,
                0.0,
                0.0,
                (settings.terrain.width - SIZE.w as u32) as f64,
            );
        } else if input.mouse_pos.x >= SIZE.w as i32 - settings.pan_edge_offset {
            self.camera.pan(
                settings.pan_speed * dt,
                0.0,
                0.0,
                (settings.terrain.width - SIZE.w as u32) as f64,
            );
        }

        // Simulate the physics
        self.physics.step(dt);

        // Update all projectiles
        self.projectiles
            .retain_mut(|projectile| projectile.update(&mut self.physics, &mut self.units, dt));

        // Update all units
        self.units.iter_mut().for_each(|unit| {
            if let Some(projectile) = unit.update(&self.terrain, dt, &mut self.physics) {
                self.projectiles.push(projectile);
            }
        });

        // Update the spawn timers and spawn a unit when it ticks
        if self.unit_spawner.update(dt) {
            // Spawn a unit at the upper edge of the terrain image
            self.units.push(Unit::new(
                Vec2::new(10.0, self.terrain.y),
                UnitType::PlayerSpear,
                &mut self.physics,
            ));
        }
        if self.enemy_unit_spawner.update(dt) {
            // Spawn a unit at the upper edge of the terrain image
            self.units.push(Unit::new(
                (settings.terrain.width as f64 - 10.0, self.terrain.y).into(),
                UnitType::EnemySpear,
                &mut self.physics,
            ));
        }

        // Update debug information
        #[cfg(feature = "debug")]
        self.debug_state.update(
            input,
            &mut self.physics,
            &mut self.projectiles,
            &mut self.terrain,
            &self.camera,
            dt,
        );
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
    /// Interval in seconds for when an enemy unit spawns.
    pub enemy_unit_spawn_interval: f64,
    /// Physics settings.
    pub physics: PhysicsSettings,
    /// Terrain settings.
    pub terrain: TerrainSettings,
    /// Debug settings.
    #[cfg(feature = "debug")]
    pub debug: DebugSettings,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

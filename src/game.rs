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
    terrain::Terrain,
    timer::Timer,
    unit::{Unit, UnitType},
    SIZE,
};

/// Physics grid step size.
const PHYSICS_GRID_STEP: u16 = 20;
/// Biggest map size.
const MAX_MAP_WIDTH: u16 = 640;
/// Maximum amount of physics objects in a single tile.
const BUCKET_SIZE: usize = 4;

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
    /// Maximum X position of the level.
    level_width: u32,
    /// Physics engine.
    ///
    /// Size of the grid is the maximum size of any map.
    physics: Physics<
        MAX_MAP_WIDTH,
        { SIZE.h as u16 },
        PHYSICS_GRID_STEP,
        BUCKET_SIZE,
        { (MAX_MAP_WIDTH / PHYSICS_GRID_STEP) as usize * (SIZE.h / PHYSICS_GRID_STEP as usize) },
    >,
    /// Debug information on the screen.
    #[cfg(feature = "debug")]
    debug_state: DebugDraw,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new() -> Self {
        let units = Vec::new();
        let unit_spawner = Timer::new(crate::settings().unit_spawn_interval);
        let enemy_unit_spawner = Timer::new(crate::settings().enemy_unit_spawn_interval);
        let projectiles = Vec::new();
        let camera = Camera::default();
        let mut physics = Physics::new();
        let terrain = Terrain::new(&mut physics);
        let level_width = terrain.width as u32;

        Self {
            projectiles,
            terrain,
            units,
            unit_spawner,
            enemy_unit_spawner,
            camera,
            level_width,
            physics,
            #[cfg(feature = "debug")]
            debug_state: DebugDraw::new(),
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], frame_time: f32) {
        crate::font("font.torus-sans").render(canvas, &format!("Castle Game: {frame_time}"), 0, 0);

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
        self.debug_state.render(canvas);
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input, dt: f32) {
        let settings = crate::settings();

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

        // Simulate the physics
        self.physics.step(dt);

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
            ));
        }
        if self.enemy_unit_spawner.update(dt) {
            // Spawn a unit at the upper edge of the terrain image
            self.units.push(Unit::new(
                (self.level_width as f32 - 10.0, self.terrain.y).into(),
                UnitType::EnemySpear,
            ));
        }

        // Update debug information
        #[cfg(feature = "debug")]
        self.debug_state.update(input, dt);
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
    /// Physics settings.
    pub physics: PhysicsSettings,
    /// Debug settings.
    #[cfg(feature = "debug")]
    pub debug: DebugSettings,
}

impl Asset for Settings {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

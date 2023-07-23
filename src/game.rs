use crate::{
    assets::Assets, camera::Camera, input::Input, terrain::Terrain, timer::Timer, unit::Unit, SIZE,
};

/// Mouse offset for panning the camera.
const PAN_EDGE_OFFSET: i32 = SIZE.w as i32 / 4;

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
        let terrain = Terrain::new(&assets.terrain_sprite);

        // Load the embedded unit
        let units = Vec::new();
        let unit_spawner = Timer::new(100.0);

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
        self.assets
            .font
            .render(canvas, &format!("Castle Game: {frame_time}"), 0, 0);

        self.terrain.render(canvas, &self.camera);

        // Render all units
        self.units
            .iter()
            .for_each(|unit| unit.render(canvas, &self.camera));
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input) {
        // Move the camera based on the mouse position
        if input.mouse_pos.x <= PAN_EDGE_OFFSET {
            self.camera
                .pan(-1.0, 0.0, 0.0, (self.level_width - SIZE.w as u32) as f64);
        } else if input.mouse_pos.x >= SIZE.w as i32 - PAN_EDGE_OFFSET {
            self.camera
                .pan(1.0, 0.0, 0.0, (self.level_width - SIZE.w as u32) as f64);
        }

        // Update all units
        self.units
            .iter_mut()
            .for_each(|unit| unit.update(&self.terrain));

        // Update the spawn timer and spawn a unit when it ticks
        if self.unit_spawner.update(1.0) {
            // Spawn a unit at the upper edge of the terrain image
            self.units.push(Unit::new(
                (10.0, self.terrain.y_offset() as f64).into(),
                self.assets,
            ));
        }
    }
}

use crate::{
    assets::Assets, camera::Camera, input::Input, terrain::Terrain,
    unit::Unit, SIZE,
};

/// Mouse offset for panning the camera.
const PAN_EDGE_OFFSET: i32 = SIZE.w as i32 / 4;

/// Handles everything related to the game.
pub struct GameState {
    /// Reference to all assets.
    assets: &'static Assets,
    /// First level ground.
    terrain: Terrain<'static>,
    /// Single unit.
    unit: Unit<'static>,
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
        let unit = Unit::new(&assets.unit_sprite, (1.0, 10.0).into());

        let level_width = terrain.width();

        let camera = Camera::default();

        Self {
            assets,
            terrain,
            unit,
            camera,
            level_width,
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32]) {
        self.assets.font.render(canvas, "Castle Game", 0, 0);

        self.terrain.render(canvas, &self.camera);
        self.unit.render(canvas, &self.camera);
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input) {
        if input.mouse_pos.x <= PAN_EDGE_OFFSET {
            self.camera
                .pan(-1.0, 0.0, 0.0, (self.level_width - SIZE.w as u32) as f64);
        } else if input.mouse_pos.x >= SIZE.w as i32 - PAN_EDGE_OFFSET {
            self.camera
                .pan(1.0, 0.0, 0.0, (self.level_width - SIZE.w as u32) as f64);
        }

        self.unit.update(&self.terrain);
    }
}

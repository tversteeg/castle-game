use crate::{camera::Camera, font::Font, input::Input, terrain::Terrain, unit::Unit, SIZE};

/// Mouse offset for panning the camera.
const PAN_EDGE_OFFSET: i32 = SIZE.w as i32 / 4;

/// Handles everything related to the game.
pub struct GameState {
    /// Font sprite.
    font: Font,
    /// First level ground.
    terrain: Terrain,
    /// Single unit.
    unit: Unit,
    /// Camera position based on the cursor.
    camera: Camera,
    /// Maximum X position of the level.
    level_width: u32,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new() -> Self {
        // Load the embedded font
        let font = Font::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/font/torus-sans.png"),
            (9, 9).into(),
        );

        // Load the embedded terrain
        let terrain = Terrain::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/level/grass-1.png"),
        );

        // Load the embedded unit
        let unit = Unit::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/unit/spear-1.png"),
            (1.0, 10.0).into(),
        );

        let level_width = terrain.width();

        let camera = Camera::default();

        Self {
            font,
            terrain,
            unit,
            camera,
            level_width,
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32]) {
        self.font.render(canvas, "Castle Game", 0, 0);

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

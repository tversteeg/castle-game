use blit::prelude::Size;

use crate::{camera::Camera, font::Font, input::Input, terrain::Terrain, SIZE};

/// Mouse offset for panning the camera.
const PAN_EDGE_OFFSET: i32 = 30;

/// Handles everything related to the game.
pub struct GameState {
    /// Font sprite.
    font: Font,
    /// First level ground.
    terrain: Terrain,
    /// Camera position based on the cursor.
    camera: Camera,
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

        let camera = Camera::default();

        Self {
            font,
            terrain,
            camera,
        }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], canvas_size: Size) {
        self.font.render(canvas, canvas_size, "Castle Game", 0, 0);

        self.terrain.render(canvas, canvas_size, &self.camera);
    }

    /// Update a frame and handle user input.
    pub fn update(&mut self, input: &Input) {
        if input.mouse_pos.x <= PAN_EDGE_OFFSET {
            self.camera.pan(1.0, 0.0);
        } else if input.mouse_pos.x >= SIZE.w as i32 - PAN_EDGE_OFFSET {
            self.camera.pan(-1.0, 0.0);
        }
    }
}

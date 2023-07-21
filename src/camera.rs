use blit::BlitOptions;

/// Camera view.
///
/// Offsets rendering.
#[derive(Default)]
pub struct Camera {
    /// X position.
    x: f64,
    /// Y position.
    y: f64,
}

impl Camera {
    /// Pan the camera.
    pub fn pan(&mut self, x: f64, y: f64) {
        self.x += x;
        self.y += y;
    }

    /// Create drawing options with the camera subrectangle to draw.
    pub fn to_blit_options(&self) -> BlitOptions {
        BlitOptions::new_position(self.x, self.y)
    }
}

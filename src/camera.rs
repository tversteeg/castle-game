use blit::BlitOptions;
use vek::Vec2;

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
    pub fn pan(&mut self, x: f64, y: f64, min_x: f64, max_x: f64) {
        self.x = (self.x + x).clamp(min_x, max_x);
        self.y += y;
    }

    /// Create drawing options with the camera subrectangle to draw.
    pub fn to_blit_options(&self) -> BlitOptions {
        BlitOptions::new_position(-self.x, -self.y)
    }

    /// Transform a world space vec2 into camera space.
    pub fn translate(&self, point: Vec2<f64>) -> Vec2<f64> {
        point - Vec2::new(self.x, self.y)
    }

    /// Transform a vec2 from screenspace into world space.
    pub fn translate_from_screen(&self, point: Vec2<f64>) -> Vec2<f64> {
        point + Vec2::new(self.x, self.y)
    }
}

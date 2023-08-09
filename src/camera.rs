use blit::BlitOptions;
use vek::Vec2;

/// Camera view.
///
/// Offsets rendering.
#[derive(Default)]
pub struct Camera {
    /// X position.
    x: f32,
    /// Y position.
    y: f32,
}

impl Camera {
    /// Pan the camera.
    pub fn pan(&mut self, x: f32, y: f32, min_x: f32, max_x: f32) {
        self.x = (self.x + x).clamp(min_x, max_x);
        self.y += y;
    }

    /// Create drawing options with the camera subrectangle to draw.
    pub fn to_blit_options(&self) -> BlitOptions {
        BlitOptions::new_position(-self.x, -self.y)
    }

    /// Transform a vec2 into camera space.
    pub fn translate(&self, point: Vec2<f32>) -> Vec2<f32> {
        point - Vec2::new(self.x, self.y)
    }
}

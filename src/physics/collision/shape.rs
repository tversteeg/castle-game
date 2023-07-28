use vek::{Aabr, Extent2, Vec2};

/// Orientable rectangle.
#[derive(Debug, Clone, Copy, Default)]
pub struct Rectangle {
    /// Half size in each direction from the center point.
    pub half_size: Extent2<f32>,
}

impl Rectangle {
    /// Construct a new rectangle with a rotation of 0.
    pub fn new(size: Extent2<f32>) -> Self {
        let half_size = size / 2.0;

        Self { half_size }
    }

    /// Calculate the axis aligned bounding rectangle.
    pub fn aabr(&self, pos: Vec2<f32>, rot: f32) -> Aabr<f32> {
        let (sin, cos) = rot.sin_cos();
        let (abs_sin, abs_cos) = (sin.abs(), cos.abs());

        let width = self.half_size.w * abs_cos + self.half_size.h * abs_sin;
        let height = self.half_size.w * abs_sin + self.half_size.h * abs_cos;
        let size = Extent2::new(width, height);

        let min = pos - size;
        let max = pos + size;

        Aabr { min, max }
    }
}

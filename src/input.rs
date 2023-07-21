use vek::Vec2;

/// Current input.
#[derive(Debug, Default)]
pub struct Input {
    pub mouse_pos: Vec2<f64>,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
}

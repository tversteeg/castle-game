use vek::Vec2;

/// Current input.
#[derive(Debug, Default)]
pub struct Input {
    pub mouse_pos: Vec2<i32>,

    pub left_mouse: ButtonState,
    pub up: ButtonState,
    pub down: ButtonState,
    pub left: ButtonState,
    pub right: ButtonState,
    pub space: ButtonState,
}

impl Input {
    /// Unset the released state.
    pub fn update(&mut self) {
        self.left_mouse.update();
        self.up.update();
        self.down.update();
        self.left.update();
        self.right.update();
        self.space.update();
    }
}

/// Input button state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonState {
    /// Button is not being pressed.
    #[default]
    None,
    /// Button was released this update tick.
    Released,
    /// Button is being pressed down.
    Pressed,
}

impl ButtonState {
    /// Whether the button is pressed.
    pub fn is_pressed(&self) -> bool {
        *self == Self::Pressed
    }

    /// Whether the button is released this tick.
    pub fn is_released(&self) -> bool {
        *self == Self::Released
    }

    /// Move state from released to none.
    pub fn update(&mut self) {
        if *self == Self::Released {
            *self = Self::None;
        }
    }

    /// Handle the window state.
    pub fn handle_bool(&mut self, pressed: bool) {
        if pressed {
            *self = Self::Pressed;
        } else if *self == Self::Pressed {
            *self = Self::Released;
        }
    }
}

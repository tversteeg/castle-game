use vek::Vec2;

use crate::{assets::Assets, camera::Camera, input::Input};

/// Draw things for debugging purposes.
#[derive(Default)]
pub struct DebugDraw {
    /// Whether to show the debug info.
    active: bool,
    /// Mouse position.
    mouse: Vec2<i32>,
}

impl DebugDraw {
    /// Update the debug state.
    pub fn update(&mut self, input: &Input) {
        if input.space_pressed {
            self.active = !self.active;
        }

        // Store the mouse state
        self.mouse = input.mouse_pos;
    }

    /// Draw things for debugging purposes.
    pub fn render(&self, canvas: &mut [u32], assets: &Assets) {
        if !self.active {
            return;
        }

        // Draw rotating sprites
        self.render_rotatable_sprite((100, 100).into(), "projectile.spear-1", canvas, assets);
    }

    /// Draw a rotatable sprite pointing towards the mouse.
    pub fn render_rotatable_sprite(
        &self,
        pos: Vec2<i32>,
        sprite_path: &str,
        canvas: &mut [u32],
        assets: &Assets,
    ) {
        let delta: Vec2<f64> = (self.mouse - pos).numcast().unwrap_or_default();
        let rotation = delta.y.atan2(delta.x);
        let sprite = assets.rotatable_sprite(sprite_path);

        sprite.render(rotation, canvas, &Camera::default(), pos);
        assets.font("font.torus-sans").render(
            canvas,
            &format!("{}", rotation.to_degrees().round()),
            pos.x,
            pos.y + 20,
        );
    }
}

use std::f64::consts::TAU;

use blit::{
    prelude::{Size, SubRect},
    Blit, BlitBuffer, BlitOptions, ToBlitBuffer,
};
use vek::Vec2;

use crate::{font::Font, input::Input, HEIGHT, WIDTH};

/// Handles everything related to the game.
pub struct GameState {
    /// Font sprite.
    font: Font,
    /// Input that can be changed by the window.
    pub input: Input,
}

impl GameState {
    /// Construct the game state with default values.
    pub fn new() -> Self {
        let font = Font::from_bytes(
            include_bytes!("../assets/font/torus-sans.png"),
            (9, 9).into(),
        );

        let input = Input::default();

        Self { font, input }
    }

    /// Draw a frame.
    pub fn render(&mut self, canvas: &mut [u32], canvas_size: Size) {
        self.font.render(canvas, canvas_size, "Castle Game", 0, 0);
    }

    /// Update a frame.
    pub fn update(&mut self) {}
}

/// Pre rendered sprites for rotations where each index is the degrees divided by the amount of rotations.
struct RotatedBlitBuffer(Vec<BlitBuffer>);

impl RotatedBlitBuffer {
    /// Rotate a buffer a set amount of times.
    pub fn from_blit_buffer(
        buffer: BlitBuffer,
        rotations: usize,
        sprite_rotation_offset: f64,
    ) -> Self {
        Self(
            (0..rotations)
                .map(|i| {
                    let (width, _, buffer) = rotsprite::rotsprite(
                        buffer.pixels(),
                        &0,
                        buffer.width() as usize,
                        i as f64 * 360.0 / rotations as f64 + sprite_rotation_offset,
                    )
                    .unwrap();

                    BlitBuffer::from_buffer(&buffer, width, 127)
                })
                .collect(),
        )
    }

    /// Draw with a set rotation around the center.
    pub fn render(&self, canvas: &mut [u32], canvas_size: Size, pos: Vec2<f64>, rotation: f64) {
        // TODO: fix rotation
        let index = (rotation / TAU * self.0.len() as f64)
            .round()
            .rem_euclid(self.0.len() as f64) as usize;

        let sprite = &self.0[index];
        sprite.blit(
            canvas,
            canvas_size,
            &BlitOptions::new_position(
                pos.x - sprite.width() as f64 / 2.0,
                pos.y - sprite.height() as f64 / 2.0,
            ),
        );
    }
}

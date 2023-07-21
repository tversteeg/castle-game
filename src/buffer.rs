use std::f64::consts::TAU;

use blit::{prelude::Size, Blit, BlitBuffer, BlitOptions};
use vek::Vec2;

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

use blit::{prelude::Size, Blit, BlitBuffer, ToBlitBuffer};
use vek::Vec2;

use crate::{camera::Camera, SIZE};

/// Sprite that can be drawn on the  canvas.
pub struct Sprite(BlitBuffer);

impl Sprite {
    /// Load a unit from image bytes.
    pub fn from_bytes(sprite_bytes: &[u8]) -> Self {
        let sprite = image::load_from_memory(sprite_bytes)
            .unwrap()
            .into_rgba8()
            .to_blit_buffer_with_alpha(127);

        Self(sprite)
    }

    /// Draw the sprite based on a camera offset.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera, offset: Vec2<i32>) {
        // Get the rendering options based on the camera offset
        let mut blit_options = camera.to_blit_options();

        // Add the additional offset
        blit_options.set_position((blit_options.x + offset.x, blit_options.y + offset.y));

        self.0.blit(canvas, SIZE.into_tuple().into(), &blit_options);
    }

    /// Whether a pixel on the image is transparent.
    pub fn is_pixel_transparent(&self, x: u32, y: u32) -> bool {
        let index = x + y * self.0.width();
        let pixel = self.0.pixels()[index as usize];

        pixel == 0
    }

    /// Width of the image.
    pub fn width(&self) -> u32 {
        self.0.width()
    }

    /// Height of the image.
    pub fn height(&self) -> u32 {
        self.0.height()
    }
}

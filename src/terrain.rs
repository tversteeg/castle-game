use blit::{prelude::Size, Blit, BlitBuffer, BlitOptions, ToBlitBuffer};
use vek::{Extent2, Vec2};

use crate::camera::Camera;

/// Destructible terrain buffer.
pub struct Terrain {
    /// Size of the terrain.
    size: Extent2<u32>,
    /// Terrain to render.
    sprite: BlitBuffer,
}

impl Terrain {
    /// Load a terrain from image bytes.
    pub fn from_bytes(sprite_bytes: &[u8]) -> Self {
        let sprite = image::load_from_memory(sprite_bytes)
            .unwrap()
            .into_rgba8()
            .to_blit_buffer_with_alpha(127);

        let size = Extent2::new(sprite.width(), sprite.height());

        Self { sprite, size }
    }

    /// Draw the terrain based on a camera offset.
    pub fn render(&self, canvas: &mut [u32], canvas_size: Size, camera: &Camera) {
        let mut blit_options = camera.to_blit_options();
        blit_options.set_position((
            blit_options.x,
            blit_options.y + canvas_size.height as i32 - self.size.h as i32,
        ));

        self.sprite.blit(canvas, canvas_size, &blit_options);
    }

    /// Width of the terrain.
    pub fn width(&self) -> u32 {
        self.size.w
    }
}

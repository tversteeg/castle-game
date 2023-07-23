use blit::{prelude::SubRect, Blit, BlitBuffer, BlitOptions, ToBlitBuffer};
use vek::Extent2;

use crate::SIZE;

/// Pixel font loaded from an image.
pub struct Font {
    /// Image to render.
    sprite: BlitBuffer,
    /// Size of a single character.
    char_size: Extent2<u8>,
}

impl Font {
    /// Load a font from image bytes.
    pub fn from_bytes(sprite_bytes: &[u8], char_size: Extent2<u8>) -> Self {
        let sprite = image::load_from_memory(sprite_bytes)
            .unwrap()
            .into_rgba8()
            .to_blit_buffer_with_mask_color(0xFF_00_FF);

        Self { sprite, char_size }
    }

    /// Render text on a pixel buffer.
    pub fn render(&self, canvas: &mut [u32], text: &str, start_x: i32, mut y: i32) {
        // First character in the image
        let char_start = '!';
        let char_end = '~';

        let mut x = start_x;

        // Draw each character from the string
        text.chars().for_each(|ch| {
            // Move the cursor
            x += self.char_size.w as i32;

            // Don't draw characters that are not in the picture
            if ch < char_start || ch > char_end {
                if ch == '\n' {
                    x = start_x;
                    y += self.char_size.h as i32;
                } else if ch == '\t' {
                    x += self.char_size.w as i32 * 3;
                }
                return;
            }

            // The sub rectangle offset of the character is based on the starting character and counted using the ASCII index
            let char_offset = (ch as u8 - char_start as u8) as u32 * self.char_size.w as u32;

            // Draw the character
            self.sprite.blit(
                canvas,
                SIZE.into_tuple().into(),
                &BlitOptions::new_position(x, y).with_sub_rect(SubRect::new(
                    char_offset,
                    0,
                    self.char_size.into_tuple(),
                )),
            );
        });
    }
}

use assets_manager::{loader::TomlLoader, AnyCache, Asset, BoxedError, Compound, SharedString};
use blit::{prelude::SubRect, Blit, BlitBuffer, BlitOptions, ToBlitBuffer};
use serde::Deserialize;
use vek::Extent2;

use crate::{sprite::Sprite, SIZE};

/// Pixel font loaded from an image.
pub struct Font {
    /// Image to render.
    sprite: BlitBuffer,
    /// Size of a single character.
    char_size: Extent2<u8>,
}

impl Font {
    /// Load a font from image bytes.
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

impl Compound for Font {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the sprite
        let sprite = cache.load_owned::<Sprite>(id)?.into_blit_buffer();

        // Load the metadata
        let metadata = cache.load::<FontMetadata>(id)?.read();
        let char_size = Extent2::new(metadata.char_width, metadata.char_height);

        Ok(Self { sprite, char_size })
    }
}

/// Font metadata to load.
#[derive(Deserialize)]
struct FontMetadata {
    /// Width of a single character.
    char_width: u8,
    /// Height of a single character.
    char_height: u8,
}

impl Asset for FontMetadata {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

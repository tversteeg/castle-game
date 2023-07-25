use std::{borrow::Cow, f64::consts::TAU, num::NonZeroU16};

use assets_manager::{
    loader::{Loader, TomlLoader},
    AnyCache, Asset, BoxedError, Compound, SharedString,
};
use blit::{Blit, BlitBuffer, ToBlitBuffer};
use image::ImageFormat;
use serde::Deserialize;
use vek::Vec2;

use crate::{camera::Camera, SIZE};

/// Sprite that can be drawn on the  canvas.
pub struct Sprite(BlitBuffer);

impl Sprite {
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

    /// Raw buffer.
    pub fn into_blit_buffer(self) -> BlitBuffer {
        self.0
    }
}

impl Asset for Sprite {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = SpriteLoader;
}

/// Sprite asset loader.
pub struct SpriteLoader;

impl Loader<Sprite> for SpriteLoader {
    fn load(content: Cow<[u8]>, _ext: &str) -> Result<Sprite, assets_manager::BoxedError> {
        let sprite = image::load_from_memory_with_format(&content, ImageFormat::Png)?
            .into_rgba8()
            .to_blit_buffer_with_alpha(127);

        Ok(Sprite(sprite))
    }
}

/// Sprite pre-rendered with different rotations.
pub struct RotatableSprite(Vec<Sprite>);

impl RotatableSprite {
    /// Create from another sprite with a set of rotations.
    ///
    /// Space between rotations is assumed to be equal in a full circle.
    pub fn with_fill_circle(
        sprite: Sprite,
        rotations: NonZeroU16,
        sprite_rotation_offset: f64,
    ) -> Self {
        let buffer = sprite.into_blit_buffer();

        Self(
            (0..rotations.get())
                .map(|i| {
                    let (width, _, buffer) = rotsprite::rotsprite(
                        buffer.pixels(),
                        &0,
                        buffer.width() as usize,
                        i as f64 * 360.0 / rotations.get() as f64 + sprite_rotation_offset,
                    )
                    .unwrap();

                    Sprite(BlitBuffer::from_buffer(&buffer, width, 127))
                })
                .collect(),
        )
    }

    /// Draw the nearest sprite based on the rotation with a camera offset.
    pub fn render(&self, rotation: f64, canvas: &mut [u32], camera: &Camera, offset: Vec2<i32>) {
        // Calculate rotation based on nearest point
        let index = (rotation / TAU * self.0.len() as f64)
            .round()
            .rem_euclid(self.0.len() as f64) as usize;

        let sprite = &self.0[index];

        // Center the sprite
        let center: Vec2<i32> = offset - (sprite.width() as i32 / 2, sprite.height() as i32 / 2);

        sprite.render(canvas, camera, center);
    }
}

impl Compound for RotatableSprite {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the sprite
        let sprite = cache.load_owned::<Sprite>(id)?;

        // Load the metadata
        let metadata = cache.load::<RotatableSpriteMetadata>(id)?.read();

        Ok(Self::with_fill_circle(sprite, metadata.rotations, 0.0))
    }
}

/// Font metadata to load.
#[derive(Deserialize)]
struct RotatableSpriteMetadata {
    /// How many rotations are pre-rendered.
    rotations: NonZeroU16,
}

impl Asset for RotatableSpriteMetadata {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

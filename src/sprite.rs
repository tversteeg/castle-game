use std::{borrow::Cow, f64::consts::TAU, num::NonZeroU16};

use assets_manager::{
    loader::{Loader, TomlLoader},
    AnyCache, Asset, BoxedError, Compound, SharedString,
};
use blit::{Blit, BlitBuffer, ToBlitBuffer};
use image::ImageFormat;
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{camera::Camera, math::Iso, SIZE};

/// Sprite that can be drawn on the  canvas.
#[derive(Debug)]
pub struct Sprite {
    /// Pixels to render.
    sprite: BlitBuffer,
    /// Pixel offset to render at.
    offset: Vec2<i32>,
}

impl Sprite {
    /// Create a sprite from a buffer of colors.
    pub fn from_buffer(buffer: &[u32], size: Extent2<usize>, offset: SpriteOffset) -> Self {
        let sprite = BlitBuffer::from_buffer(buffer, size.w, 127);
        let offset = offset.offset(size.as_());

        Self { sprite, offset }
    }

    /// Draw the sprite based on a camera offset.
    pub fn render(&self, canvas: &mut [u32], camera: &Camera, offset: Vec2<f64>) {
        puffin::profile_function!();

        // Get the rendering options based on the camera offset
        let mut blit_options = camera.to_blit_options();
        let offset: Vec2<i32> = offset.as_() + self.offset.as_();

        // Add the additional offset
        blit_options.set_position((blit_options.x + offset.x, blit_options.y + offset.y));

        self.sprite
            .blit(canvas, SIZE.into_tuple().into(), &blit_options);
    }

    /// Whether a pixel on the image is transparent.
    pub fn is_pixel_transparent(&self, pixel: Vec2<u32>) -> bool {
        let offset: Vec2<i32> = pixel.as_() + self.offset;

        let index: i32 = offset.x + offset.y * self.sprite.width() as i32;
        let pixel = self.sprite.pixels()[index as usize];

        pixel == 0
    }

    /// Width of the image.
    pub fn width(&self) -> u32 {
        self.sprite.width()
    }

    /// Height of the image.
    pub fn height(&self) -> u32 {
        self.sprite.height()
    }

    /// Size of the image.
    pub fn size(&self) -> Extent2<u32> {
        Extent2::new(self.width(), self.height())
    }

    /// Raw buffer.
    pub fn into_blit_buffer(self) -> BlitBuffer {
        self.sprite
    }

    /// Get the raw pixels.
    pub fn pixels_mut(&mut self) -> &mut [u32] {
        self.sprite.pixels_mut()
    }

    /// Calculate the highest non-transparent pixels of an image.
    pub fn top_heights(&self) -> Vec<u16> {
        puffin::profile_scope!("Sprite top heights");

        (0..self.width())
            .map(|x| {
                // Loop over each Y value to find the first pixel that is not transparent
                (0..self.height())
                    .find(|y| !self.is_pixel_transparent(Vec2::new(x, *y)))
                    // If nothing is found just use the bottom
                    .unwrap_or(self.height()) as u16
            })
            .collect()
    }
}

impl Asset for Sprite {
    // We only support PNG images currently
    const EXTENSION: &'static str = "png";

    type Loader = SpriteLoader;
}

impl Default for Sprite {
    fn default() -> Self {
        let sprite = BlitBuffer::from_buffer(&[0], 1, 0);
        let offset = Vec2::zero();

        Self { sprite, offset }
    }
}

/// Sprite asset loader.
pub struct SpriteLoader;

impl Loader<Sprite> for SpriteLoader {
    fn load(content: Cow<[u8]>, _ext: &str) -> Result<Sprite, assets_manager::BoxedError> {
        let sprite = image::load_from_memory_with_format(&content, ImageFormat::Png)?
            .into_rgba8()
            .to_blit_buffer_with_alpha(127);

        let offset = Vec2::zero();

        Ok(Sprite { sprite, offset })
    }
}

/// Sprite pre-rendered with different rotations.
#[derive(Debug)]
pub struct RotatableSprite(Vec<Sprite>);

impl RotatableSprite {
    /// Create from another sprite with a set of rotations.
    ///
    /// Space between rotations is assumed to be equal in a full circle.
    pub fn with_fill_circle(
        sprite: Sprite,
        metadata: RotatableSpriteMetadata,
        sprite_rotation_offset: f64,
    ) -> Self {
        let buffer = sprite.into_blit_buffer();

        let rotations = metadata.rotations.get();
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

                    let sprite = BlitBuffer::from_buffer(&buffer, width, 127);

                    // TODO: factor in rotations
                    let offset = metadata
                        .offset
                        .offset(Extent2::new(sprite.width(), sprite.height()));

                    Sprite { sprite, offset }
                })
                .collect(),
        )
    }

    /// Draw the nearest sprite based on the rotation with a camera offset.
    pub fn render(&self, iso: Iso, canvas: &mut [u32], camera: &Camera) {
        let rotation = iso.rot.to_radians();

        // Calculate rotation based on nearest point
        let index = (rotation / TAU * self.0.len() as f64)
            .round()
            .rem_euclid(self.0.len() as f64) as usize;

        let sprite = &self.0[index];

        sprite.render(canvas, camera, iso.pos);
    }
}

impl Compound for RotatableSprite {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the sprite
        let sprite = cache.load_owned::<Sprite>(id)?;

        // Load the metadata
        let metadata = cache.load::<RotatableSpriteMetadata>(id)?.read();

        Ok(Self::with_fill_circle(sprite, *metadata, 0.0))
    }
}

/// Center of the sprite.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SpriteOffset {
    /// Middle of the sprite will be rendered at `(0, 0)`.
    #[default]
    Middle,
    /// Horizontal middle and vertical top will be rendered at `(0, 0)`.
    MiddleTop,
    /// Left top of the sprite will be rendered at `(0, 0)`.
    LeftTop,
    /// Sprite will be offset with the custom coordinates counting from the left top.
    Custom(Vec2<u32>),
}

impl SpriteOffset {
    /// Get the offset based on the sprite size.
    pub fn offset(&self, sprite_size: Extent2<u32>) -> Vec2<i32> {
        match self {
            SpriteOffset::Middle => {
                Vec2::new(-(sprite_size.w as i32) / 2, -(sprite_size.h as i32) / 2)
            }
            SpriteOffset::MiddleTop => Vec2::new(-(sprite_size.w as i32) / 2, 0),
            SpriteOffset::LeftTop => Vec2::zero(),
            SpriteOffset::Custom(offset) => offset.as_(),
        }
    }
}

/// Rotatable sprite metadata to load.
#[derive(Deserialize, Clone, Copy)]
pub struct RotatableSpriteMetadata {
    /// How many rotations are pre-rendered.
    rotations: NonZeroU16,
    /// Center of where sprite will be rendered.
    #[serde(default)]
    offset: SpriteOffset,
}

impl Asset for RotatableSpriteMetadata {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

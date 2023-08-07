use std::ops::Deref;

use assets_manager::{
    loader::{Loader, TomlLoader},
    AnyCache, Asset, BoxedError, Compound, SharedString,
};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{
    physics::{collision::shape::Shape, rigidbody::RigidBody},
    sprite::Sprite,
};

/// Loadable object with physics.
#[derive(Deserialize)]
pub struct ObjectSettings(ObjectSettingsImpl);

impl Deref for ObjectSettings {
    type Target = ObjectSettingsImpl;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Compound for ObjectSettings {
    fn load(cache: AnyCache, id: &SharedString) -> Result<Self, BoxedError> {
        // Load the settings
        let mut settings = cache.load_owned::<ObjectSettingsImpl>(id)?;

        // Set the values from the attached sprite path if any of the sizes has a mismatch
        match &mut settings.collider {
            ColliderSettings::Rectangle {
                ref mut width,
                ref mut height,
            } => {
                if *width == 0.0 || *height == 0.0 {
                    let sprite = cache.load::<Sprite>(id)?.read();
                    if *width == 0.0 {
                        *width = sprite.width() as f32;
                    }
                    if *height == 0.0 {
                        *height = sprite.height() as f32;
                    }
                }
            }
            // Generate a heightmap from the sprite
            ColliderSettings::Heightmap {
                spacing,
                height_offset,
                ref mut heights,
            } => {
                let sprite = cache.load::<Sprite>(id)?.read();
                let amount_heights = sprite.width() / *spacing as u32;

                // Calculate the new heights from the sprite
                *heights = (0..amount_heights)
                    .map(|index| {
                        let x = index * *spacing as u32;

                        (0..sprite.height())
                            // Find the top pixel that's non-transparent as the top of the heigthfield
                            .find(|y| !sprite.is_pixel_transparent(Vec2::new(x, *y)))
                            .unwrap_or(sprite.height()) as f32
                            + *height_offset
                    })
                    .collect();
            }
        };

        Ok(Self(settings))
    }
}

/// Internal data so we can load it as a compound.
#[derive(Debug, Deserialize)]
pub struct ObjectSettingsImpl {
    /// Physics information.
    physics: PhysicsSettings,
    /// Collider information.
    collider: ColliderSettings,
}

impl ObjectSettingsImpl {
    /// Construct a rigidbody from the metadata.
    pub fn rigidbody(&self, pos: Vec2<f32>) -> RigidBody {
        RigidBody::new(pos, self.physics.density, self.shape())
    }

    /// Construct a collider shape from the metadata.
    pub fn shape(&self) -> Shape {
        match &self.collider {
            ColliderSettings::Rectangle { width, height } => {
                Shape::rectangle(Extent2::new(*width, *height))
            }
            ColliderSettings::Heightmap {
                spacing, heights, ..
            } => Shape::heightmap(heights, *spacing as f32),
        }
    }

    /// Width of the shape.
    pub fn width(&self) -> f32 {
        match self.collider {
            ColliderSettings::Rectangle { width, .. } => width,
            ColliderSettings::Heightmap {
                spacing,
                ref heights,
                ..
            } => heights.len() as f32 * spacing as f32,
        }
    }

    /// Height of the shape.
    pub fn height(&self) -> f32 {
        match self.collider {
            ColliderSettings::Rectangle { height, .. } => height,
            // Cannot be known
            ColliderSettings::Heightmap { .. } => 0.0,
        }
    }
}

impl Asset for ObjectSettingsImpl {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

/// Physics settings for a rigid body.
#[derive(Debug, Deserialize)]
struct PhysicsSettings {
    /// Mass is density times area.
    density: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self { density: 1.0 }
    }
}

/// Collider settings for a rigid body.
#[derive(Debug, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
enum ColliderSettings {
    Rectangle {
        /// Width of the collider, if `0.0` the sprite size is used.
        #[serde(default)]
        width: f32,
        /// Height of the collider, if `0.0` the sprite size is used.
        #[serde(default)]
        height: f32,
    },
    Heightmap {
        /// How many X pixels will be skipped before the next sample is taken.
        spacing: u8,
        /// How much height below a pixel is used for the collision shape.
        #[serde(default)]
        height_offset: f32,
        /// List of heights, will be calculated from the image.
        #[serde(default)]
        heights: Vec<f32>,
    },
}

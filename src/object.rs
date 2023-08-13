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
                        *width = sprite.width() as f64;
                    }
                    if *height == 0.0 {
                        *height = sprite.height() as f64;
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

                assert_eq!(
                    sprite.width() % *spacing,
                    0,
                    "Spacing of heightmap must be divisible by sprite width"
                );
                let amount_heights = sprite.width() / *spacing;

                // Calculate the new heights from the sprite
                *heights = (0..amount_heights)
                    .map(|index| {
                        let x = index * *spacing;

                        (0..sprite.height())
                            // Find the top pixel that's non-transparent as the top of the heigthfield
                            .find(|y| !sprite.is_pixel_transparent(Vec2::new(x, *y)))
                            .unwrap_or(sprite.height()) as f64
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
    pub fn rigidbody(&self, pos: Vec2<f64>) -> RigidBody {
        if self.physics.is_fixed {
            RigidBody::new_fixed(pos, self.shape())
        } else {
            RigidBody::new(pos, self.shape())
                .with_density(self.physics.density)
                .with_friction(self.physics.friction)
                .with_restitution(self.physics.restitution)
                .with_linear_damping(self.physics.linear_damping)
                .with_angular_damping(self.physics.angular_damping)
        }
    }

    /// Construct a collider shape from the metadata.
    pub fn shape(&self) -> Shape {
        match &self.collider {
            ColliderSettings::Rectangle { width, height } => {
                Shape::rectangle(Extent2::new(*width, *height))
            }
            ColliderSettings::Heightmap {
                spacing, heights, ..
            } => Shape::heightmap(heights, *spacing as f64),
        }
    }
}

impl Asset for ObjectSettingsImpl {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

/// Physics settings for a rigid body.
#[derive(Debug, Deserialize)]
#[serde(default)]
struct PhysicsSettings {
    /// Whether this is a fixed object, means it can't move.
    is_fixed: bool,
    /// Mass is density times area.
    ///
    /// Doesn't apply when this is a static object.
    density: f64,
    /// Friction coefficient for both static and dynamic friction.
    friction: f64,
    /// Restitution coefficiont for bounciness.
    restitution: f64,
    /// Linear damping.
    ///
    /// Doesn't apply when this is a static object.
    linear_damping: f64,
    /// Angular damping.
    ///
    /// Doesn't apply when this is a static object.
    angular_damping: f64,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            is_fixed: false,
            density: 1.0,
            friction: 0.3,
            restitution: 0.3,
            linear_damping: 1.0,
            angular_damping: 1.0,
        }
    }
}

/// Collider settings for a rigid body.
#[derive(Debug, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
enum ColliderSettings {
    Rectangle {
        /// Width of the collider, if `0.0` the sprite size is used.
        #[serde(default)]
        width: f64,
        /// Height of the collider, if `0.0` the sprite size is used.
        #[serde(default)]
        height: f64,
    },
    Heightmap {
        /// How many X pixels will be skipped before the next sample is taken.
        ///
        /// Width must be divisible by the spacing.
        spacing: u32,
        /// How much height below a pixel is used for the collision shape.
        #[serde(default)]
        height_offset: f64,
        /// List of heights, will be calculated from the image.
        #[serde(default)]
        heights: Vec<f64>,
    },
}

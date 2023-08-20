use assets_manager::{loader::TomlLoader, AnyCache, Asset, BoxedError, Compound, SharedString};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{
    physics::{collision::shape::Shape, rigidbody::RigidBodyBuilder},
    sprite::Sprite,
};

/// Loadable object with physics.
pub struct ObjectSettings {
    settings: ObjectSettingsImpl,
    /// Pre-computed shape that can be cloned.
    ///
    /// This is good for performance because parry shared shapes are shared by a reference counter.
    shape: Shape,
}

impl ObjectSettings {
    /// Construct a rigidbody from the metadata.
    pub fn rigidbody_builder(&self, pos: Vec2<f64>) -> RigidBodyBuilder {
        if self.settings.physics.is_fixed {
            RigidBodyBuilder::new_static(pos).with_collider(self.shape())
        } else {
            let builder = if !self.settings.physics.is_kinematic {
                RigidBodyBuilder::new(pos)
            } else {
                RigidBodyBuilder::new_kinematic(pos)
            }
            .with_collider(self.shape());

            let builder = if let Some(density) = self.settings.physics.density {
                builder.with_density(density)
            } else {
                builder
            };
            let builder = if let Some(friction) = self.settings.physics.friction {
                builder.with_friction(friction)
            } else {
                builder
            };
            let builder = if let Some(restitution) = self.settings.physics.restitution {
                builder.with_restitution(restitution)
            } else {
                builder
            };
            let builder = if let Some(linear_damping) = self.settings.physics.linear_damping {
                builder.with_linear_damping(linear_damping)
            } else {
                builder
            };
            if let Some(angular_damping) = self.settings.physics.angular_damping {
                builder.with_angular_damping(angular_damping)
            } else {
                builder
            }
        }
    }

    /// Copy a shape reference.
    pub fn shape(&self) -> Shape {
        self.shape.clone()
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

        let shape = settings.construct_shape();

        Ok(Self { shape, settings })
    }
}

/// Internal data so we can load it as a compound.
#[derive(Debug, Deserialize)]
pub struct ObjectSettingsImpl {
    /// Physics information.
    #[serde(default)]
    physics: PhysicsSettings,
    /// Collider information.
    collider: ColliderSettings,
}

impl ObjectSettingsImpl {
    /// Construct a collider shape from the metadata.
    pub fn construct_shape(&self) -> Shape {
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
#[derive(Debug, Default, Deserialize)]
#[serde(default)]
struct PhysicsSettings {
    /// Whether this is a fixed object, means it can't move and has infinite mass.
    is_fixed: bool,
    /// Whether this is a kinematic object, means collisions don't influence it.
    is_kinematic: bool,
    /// Mass is density times area.
    ///
    /// Doesn't apply when this is a static object.
    density: Option<f64>,
    /// Friction coefficient for both static and dynamic friction.
    friction: Option<f64>,
    /// Restitution coefficiont for bounciness.
    restitution: Option<f64>,
    /// Linear damping.
    ///
    /// Doesn't apply when this is a static object.
    linear_damping: Option<f64>,
    /// Angular damping.
    ///
    /// Doesn't apply when this is a static object.
    angular_damping: Option<f64>,
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

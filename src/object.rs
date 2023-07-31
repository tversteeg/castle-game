use std::{borrow::Cow, ops::Deref};

use assets_manager::{
    loader::{Loader, TomlLoader},
    AnyCache, Asset, BoxedError, Compound, SharedString,
};
use serde::Deserialize;
use vek::{Extent2, Vec2};

use crate::{
    physics::{collision::shape::Rectangle, rigidbody::RigidBody},
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
        };

        Ok(Self(settings))
    }
}

/// Internal data so we can load it as a compound.
#[derive(Deserialize)]
pub struct ObjectSettingsImpl {
    /// Physics information.
    physics: PhysicsSettings,
    /// Collider information.
    collider: ColliderSettings,
}

impl ObjectSettingsImpl {
    /// Construct a rigidbody from the metadata.
    pub fn rigidbody(&self, pos: Vec2<f32>) -> RigidBody {
        RigidBody::new(pos, self.physics.mass, self.shape())
    }

    /// Construct a collider shape from the metadata.
    pub fn shape(&self) -> Rectangle {
        match self.collider {
            ColliderSettings::Rectangle { width, height } => {
                Rectangle::new(Extent2::new(width, height))
            }
        }
    }

    /// Width of the shape.
    pub fn width(&self) -> f32 {
        match self.collider {
            ColliderSettings::Rectangle { width, .. } => width,
        }
    }

    /// Height of the shape.
    pub fn height(&self) -> f32 {
        match self.collider {
            ColliderSettings::Rectangle { height, .. } => height,
        }
    }
}

impl Asset for ObjectSettingsImpl {
    const EXTENSION: &'static str = "toml";

    type Loader = TomlLoader;
}

/// Physics settings for a rigid body.
#[derive(Deserialize)]
struct PhysicsSettings {
    /// Mass of the body.
    mass: f32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self { mass: 1.0 }
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
}

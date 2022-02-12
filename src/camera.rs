use crate::projectile::arrow::Arrow;
use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Camera object that is controlled by the user.
#[derive(Component)]
pub struct MainCamera {
    /// Speed with which to move to the object.
    speed_factor: f32,
}

impl Default for MainCamera {
    fn default() -> Self {
        Self { speed_factor: 0.2 }
    }
}

pub fn move_camera(
    arrow: Query<&RigidBodyPositionComponent, With<Arrow>>,
    mut camera: Query<(&mut Transform, &MainCamera)>,
) {
    // Get the position of the arrow
    let position = arrow.single();
    let arrow_translation = &position.position.translation;

    // Get the transform of the main camera
    let (mut camera_transform, main_camera): (Mut<Transform>, &MainCamera) = camera.single_mut();

    // Move towards the arrow
    camera_transform.translation.x +=
        (arrow_translation.x - camera_transform.translation.x) * main_camera.speed_factor;
    camera_transform.translation.y +=
        (arrow_translation.y - camera_transform.translation.y) * main_camera.speed_factor;
}

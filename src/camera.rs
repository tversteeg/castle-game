use bevy::{
    math::Vec3,
    prelude::{App, Commands, OrthographicCameraBundle, Plugin, Transform, UiCameraBundle},
};

/// How far the camera is zoomed in.
pub const CAMERA_SCALE: f32 = 1.0 / 10.0;

/// The plugin to handle camera movements.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}

/// Initial setup for the camera.
fn setup(mut commands: Commands) {
    // Setup the cameras
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform {
        scale: Vec3::splat(CAMERA_SCALE),
        ..Default::default()
    };
    commands.spawn_bundle(camera);
    commands.spawn_bundle(UiCameraBundle::default());
}

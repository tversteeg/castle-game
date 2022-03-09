use bevy::{
    core::Name,
    math::Vec3,
    prelude::{
        App, Commands, Component, EventReader, GlobalTransform, OrthographicCameraBundle, Plugin,
        Query, Res, Transform, UiCameraBundle, With,
    },
    render::camera::WindowOrigin,
    window::{CursorMoved, Windows},
};

use crate::map::terrain::TERRAIN_WIDTH;

/// How far the camera is zoomed in.
pub const CAMERA_SCALE: f32 = 1.0 / 10.0;
/// Camera border on the each on which it won't move.
pub const CAMERA_BORDER_SIZE: f32 = 0.2;

/// The plugin to handle camera movements.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(system);
    }
}

/// The component we can attach to the camera.
#[derive(Component)]
pub struct Camera;

/// Initial setup for the camera.
fn setup(mut commands: Commands) {
    // Setup the cameras
    let mut camera = OrthographicCameraBundle::new_2d();

    camera.transform = Transform {
        scale: Vec3::splat(CAMERA_SCALE),
        ..Default::default()
    };

    camera.orthographic_projection.window_origin = WindowOrigin::BottomLeft;

    // Draw everything with z-index 0.0..100.0
    camera.orthographic_projection.near = -1000.0;

    commands
        .spawn_bundle(camera)
        .insert(Camera)
        .insert(Name::new("Camera"));
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(Name::new("UI Camera"));
}

/// The system for following the mouse with the camera.
pub fn system(
    mut events: EventReader<CursorMoved>,
    windows: Res<Windows>,
    mut query: Query<&mut GlobalTransform, With<Camera>>,
) {
    events.iter().for_each(|event| {
        // The camera should always be in the query
        let mut transform = query.iter_mut().next().unwrap();

        // Get the window size so we can calculate the max camera position
        let window_size = windows.get(event.id).unwrap();

        // The maximum position of the camera to the right
        let max_position = TERRAIN_WIDTH - window_size.width() * CAMERA_SCALE;

        // The position of the mouse as a fraction
        // Keep a zone on the edges in which moving the mouse won't move the camera
        let mouse_x = ((event.position.x / window_size.width()) * (1.0 + CAMERA_BORDER_SIZE * 2.0)
            - CAMERA_BORDER_SIZE)
            .clamp(0.0, 1.0);

        // Position the camera at the mouse
        transform.translation = Vec3::new(mouse_x * max_position, 0.0, 0.0);
    });
}

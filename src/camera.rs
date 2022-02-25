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

/// How far the camera is zoomed in.
pub const CAMERA_SCALE: f32 = 1.0 / 10.0;

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

        // TODO: make maximum camera pan the window size
        let _window_size = windows.get(event.id).unwrap();

        // Position the camera at the mouse
        transform.translation = Vec3::new(
            event.position.x / 10.0,
            //(-window_size.height() / 2.0 + event.position.y) / 10.0,
            0.0,
            0.0,
        );
    });
}

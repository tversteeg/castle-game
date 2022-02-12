mod camera;
mod map;
mod physics;
mod projectile;

use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_rapier2d::{
    physics::{NoUserData, RapierConfiguration, RapierPhysicsPlugin},
    render::RapierRenderPlugin,
};
use camera::MainCamera;

/// How far the camera is zoomed in.
const CAMERA_SCALE: f32 = 1.0 / 10.0;

fn main() {
    App::new()
        // Setup the window
        .insert_resource(WindowDescriptor {
            width: 300.0,
            height: 300.0,
            title: "Castle Game".to_string(),
            ..Default::default()
        })
        // Setup the physics, units used are m/s
        .insert_resource(RapierConfiguration {
            gravity: [0.0, -9.2].into(),
            ..Default::default()
        })
        // Default, needed for physics
        .add_plugins(DefaultPlugins)
        // Physics
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_plugin(EasingsPlugin)
        .add_startup_system(map::setup_ground.system())
        .add_startup_system(setup)
        // Close when Esc is pressed
        .add_system(bevy::input::system::exit_on_esc_system)
        // The main camera
        .add_system(camera::move_camera)
        .run();
}

/// Setup the basic bundles and objects.
fn setup(mut commands: Commands) {
    // Setup the cameras
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.transform = Transform {
        scale: Vec3::splat(CAMERA_SCALE),
        ..Default::default()
    };
    commands.spawn_bundle(camera).insert(MainCamera::default());
    commands.spawn_bundle(UiCameraBundle::default());

    // Setup a light point
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(1000.0, 10.0, 2000.0)),
        point_light: PointLight {
            intensity: 100_000_000_.0,
            range: 6000.0,
            ..Default::default()
        },
        ..Default::default()
    });

    projectile::arrow::spawn([0.0, 0.0].into(), [0.82, 0.0].into(), commands);
}

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

/// Setup the ground physics system.
pub fn setup_ground(mut commands: Commands) {
    let size = Vec2::new(100.0, 1.2);

    // Create the ground box
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: size.into(),
                color: Color::rgb(0.0, 0.6, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(size.x / 2.0, size.y / 2.0).into(),
            position: [size.x / 2.0, -100.0].into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete);
}

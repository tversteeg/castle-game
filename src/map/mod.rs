use bevy::prelude::*;

/// Setup the ground physics system.
pub fn setup_ground(mut commands: Commands) {
    let size = Vec2::new(100.0, 1.2);

    // Create the ground box
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            custom_size: size.into(),
            color: Color::rgb(0.0, 0.6, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

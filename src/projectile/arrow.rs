use crate::projectile::Projectile;
use bevy::{
    math::Vec2,
    prelude::{Color, Commands, Component},
    sprite::{Sprite, SpriteBundle},
};
use crate::inspector::Inspectable;

#[derive(Component, Inspectable)]
pub struct Arrow;

/// Shoot a new arrow.
pub fn spawn(commands: &mut Commands) {
    // The average arrow is 64cm long
    let size = Vec2::new(0.64, 0.05);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: size.into(),
                color: Color::rgb(1.0, 0.5, 0.5),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Projectile)
        .insert(Arrow);
}

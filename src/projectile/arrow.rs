use bevy::{
    math::{Vec2, Vec3},
    prelude::{Color, Commands, Component, Transform},
    sprite::{Sprite, SpriteBundle},
};
use bevy_rapier2d::prelude::*;

#[derive(Component)]
pub struct Arrow;

/// Shoot a new arrow.
pub fn spawn(position: Vec2, velocity: Vec2, mut commands: Commands) {
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
        .insert_bundle(RigidBodyBundle {
            position: position.into(),
            velocity: RigidBodyVelocity {
                linvel: velocity.into(),
                angvel: 0.0,
            }
            .into(),
            ..Default::default()
        })
        .insert_bundle(ColliderBundle {
            shape: ColliderShape::cuboid(size.x / 2.0, size.y / 2.0).into(),
            ..Default::default()
        })
        .insert(ColliderPositionSync::Discrete)
        .insert(Arrow);
}

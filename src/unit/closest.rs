use super::{faction::Faction, unit_type::UnitType};
use crate::inspector::Inspectable;
use bevy::prelude::{Query, ResMut, Transform, With};

/// The closest enemy.
#[derive(Debug, Default, Inspectable)]
pub struct ClosestEnemy {
    /// The X position on the map.
    pub x: Option<f32>,
}

impl ClosestEnemy {
    /// Get the X position of the next enemy or infinite.
    pub fn x_or_inf(&self) -> f32 {
        self.x.unwrap_or(f32::MAX)
    }
}

/// The closest ally.
#[derive(Debug, Default, Inspectable)]
pub struct ClosestAlly {
    /// The X position on the map.
    pub x: Option<f32>,
}

impl ClosestAlly {
    /// Get the X position of the next enemy or infinite.
    pub fn x_or_inf(&self) -> f32 {
        self.x.unwrap_or(f32::MIN)
    }
}

/// Update the closest positions resources.
pub fn system(
    mut closest_enemy: ResMut<ClosestEnemy>,
    mut closest_ally: ResMut<ClosestAlly>,
    query: Query<(&Faction, &Transform), With<UnitType>>,
) {
    closest_ally.x = None;
    closest_enemy.x = None;

    for (faction, transform) in query.iter() {
        match faction {
            Faction::Ally => {
                if transform.translation.x > closest_ally.x_or_inf() {
                    closest_ally.x = Some(transform.translation.x);
                }
            }
            Faction::Enemy => {
                if transform.translation.x < closest_enemy.x_or_inf() {
                    closest_enemy.x = Some(transform.translation.x);
                }
            }
        }
    }
}

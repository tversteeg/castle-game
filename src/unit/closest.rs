use super::faction::Faction;
use bevy::prelude::{Query, ResMut, Transform};
use crate::inspector::Inspectable;

/// The closest enemy.
#[derive(Debug, Default, Inspectable)]
pub struct ClosestEnemy {
    /// The X position on the map.
    pub x: Option<f32>,
}

/// The closest ally.
#[derive(Debug, Default, Inspectable)]
pub struct ClosestAlly {
    /// The X position on the map.
    pub x: Option<f32>,
}

/// Update the closest positions resources.
pub fn system(
    mut closest_enemy: ResMut<ClosestEnemy>,
    mut closest_ally: ResMut<ClosestAlly>,
    query: Query<(&Faction, &Transform)>,
) {
    closest_ally.x = None;
    closest_enemy.x = None;

    for (faction, transform) in query.iter() {
        match faction {
            Faction::Ally => {
                if transform.translation.x > closest_ally.x.unwrap_or(f32::MIN) {
                    closest_ally.x = Some(transform.translation.x);
                }
            }
            Faction::Enemy => {
                if transform.translation.x < closest_enemy.x.unwrap_or(f32::MAX) {
                    closest_enemy.x = Some(transform.translation.x);
                }
            }
        }
    }
}

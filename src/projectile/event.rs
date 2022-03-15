use bevy::math::Vec2;

/// What type of projectile to spawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileType {
    /// Just do direct damage.
    Direct,
    /// Spawn an arrow.
    Arrow,
    /// Spawn a rock.
    Rock,
}

/// The event that's fired when a projectile needs to be spawned.
#[derive(Debug)]
pub struct ProjectileSpawnEvent {
    /// What type of projectile to spawn.
    pub projectile_type: ProjectileType,
    /// Where the projectile should be spawned.
    pub start_position: Vec2,
    /// Where the projectile should fly to.
    pub target_position: Option<Vec2>,
}

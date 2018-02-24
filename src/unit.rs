use specs::*;

use super::*;

#[derive(Component, Debug, Eq, PartialEq)]
pub enum UnitState {
    Walk,
    Melee,
    Shoot
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Health(pub f64);

#[derive(Component, Debug, Copy, Clone)]
pub struct Walk {
    pub bounds: BoundingBox,
    pub speed: f64,
}

impl Walk {
    pub fn new(bounds: BoundingBox, speed: f64) -> Self {
        Walk { bounds, speed, }
    }
}

pub struct WalkSystem;
impl<'a> System<'a> for WalkSystem {
    type SystemData = (Fetch<'a, DeltaTime>,
                       Fetch<'a, Terrain>,
                       ReadStorage<'a, Destination>,
                       ReadStorage<'a, Walk>,
                       WriteStorage<'a, UnitState>,
                       WriteStorage<'a, WorldPosition>);

    fn run(&mut self, (dt, terrain, dest, walk, mut state, mut pos): Self::SystemData) {
        let dt = dt.to_seconds();

        for (dest, walk, state, pos) in (&dest, &walk, &mut state, &mut pos).join() {
            pos.0.y += 1.0;

            loop {
                let hit_box = walk.bounds + *pos.0;
                match terrain.rect_collides(hit_box) {
                    Some(hit) => {
                        pos.0.y -= 1.0;

                        // Don't walk when the unitstate is not saying that it can walk
                        if *state != UnitState::Walk {
                            *state = UnitState::Walk;
                            break;
                        }

                        if hit.1 == hit_box.min.y as i32 {
                            // Top edge of bounding box is hit, don't walk anymore
                            break;
                        }

                        pos.0.x += walk.speed * dt * (dest.0 - pos.0.x).signum();
                    },
                    None => break
                }
            }
        }
    }
}

pub fn reduce_unit_health<'a>(entities: &'a EntitiesRes, unit: &'a Entity, health: &'a mut Health, dmg: f64) -> bool {
    health.0 -= dmg;
    if health.0 <= 0.0 {
        let _ = entities.delete(*unit);

        return true;
    }

    return false;
}

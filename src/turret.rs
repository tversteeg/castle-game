use specs::*;
use rand;
use rand::distributions::{IndependentSample, Range};
use cgmath::MetricSpace;

use super::*;

#[derive(Component, Debug)]
pub struct Turret {
    pub delay: f64,
    pub min_distance: f64,
    pub max_strength: f64,
    pub flight_time: f64,
    pub strength_variation: f64,

    pub delay_left: f64
}

impl Default for Turret {
    fn default() -> Self {
        Turret {
            delay: 5.0,
            min_distance: 20.0,
            max_strength: 210.0,
            flight_time: 3.0,
            strength_variation: 0.1,

            delay_left: 0.0
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct TurretOffset(pub (f64, f64));

pub struct TurretUnitSystem;
impl<'a> System<'a> for TurretUnitSystem {
    type SystemData = (ReadStorage<'a, Turret>,
                       ReadStorage<'a, WorldPosition>,
                       ReadStorage<'a, TurretOffset>,
                       WriteStorage<'a, UnitState>,
                       WriteStorage<'a, Point>);

    fn run(&mut self, (turret, wpos, offset, mut state, mut pos): Self::SystemData) {
        for (turret, wpos, offset, state, pos) in (&turret, &wpos, &offset, &mut state, &mut pos).join() {
            pos.0.x = wpos.0.x + (offset.0).0;
            pos.0.y = wpos.0.y + (offset.0).1;

            let unit_stop_moving_offset = turret.delay / 4.0;
            if turret.delay_left > unit_stop_moving_offset {
                // Set the state if the unit turret just shot
                *state = UnitState::Shoot;
            } else if turret.delay_left < unit_stop_moving_offset && *state == UnitState::Shoot {
                // Make the unit walk again when the timeout is over
                *state = UnitState::Walk;
            }
        }
    }
}

pub struct TurretSystem;
impl<'a> System<'a> for TurretSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       ReadStorage<'a, Ally>,
                       ReadStorage<'a, Enemy>,
                       ReadStorage<'a, Point>,
                       ReadStorage<'a, WorldPosition>,
                       ReadStorage<'a, ProjectileSprite>,
                       ReadStorage<'a, Arrow>,
                       ReadStorage<'a, Line>,
                       ReadStorage<'a, MaskId>,
                       ReadStorage<'a, IgnoreCollision>,
                       ReadStorage<'a, ProjectileBoundingBox>,
                       ReadStorage<'a, BoundingBox>,
                       ReadStorage<'a, Damage>,
                       ReadStorage<'a, Walk>,
                       ReadStorage<'a, UnitState>,
                       WriteStorage<'a, Turret>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, grav, ally, enemy, pos, wpos, sprite, arrow, line, mask, ignore, bb, ubb, dmg, walk, state, mut turret, updater): Self::SystemData) {
        let dt = dt.to_seconds();
        let grav = grav.0;

        for (e, tpos, bb, dmg, turret) in (&*entities, &pos, &bb, &dmg, &mut turret).join() {
            turret.delay_left -= dt;
            if turret.delay_left > 0.0 {
                continue;
            }

            // Find the nearest ally to shoot
            let mut closest = Point::new(-1000.0, -1000.0);
            let mut dist = tpos.distance(*closest);

            let is_ally: Option<&Ally> = ally.get(e);
            if let Some(_) = is_ally {
                for (epos, _, walk, ubb, state) in (&wpos, &enemy, &walk, &ubb, &state).join() {
                    let mut pos = epos.0;
                    if *state == UnitState::Walk {
                        pos.x += walk.speed * turret.flight_time;
                    }

                    let dist_to = tpos.distance(*pos);
                    if dist_to < dist && dist_to > turret.min_distance {
                        dist = dist_to;
                        closest = pos;

                        // TODO Make the projectile hit the center of the target instead of the zero
                        // point
                        //closest.x += ubb.width() / 2.0 + (epos.0.x - ubb.x());
                        //closest.y += ubb.height() / 2.0 + (epos.0.y - ubb.y());
                    }
                }
            } else {
                for (apos, _, walk, ubb, state) in (&wpos, &ally, &walk, &ubb, &state).join() {
                    let mut pos = apos.0;
                    if *state == UnitState::Walk {
                        pos.x += walk.speed * turret.flight_time;
                    }

                    let dist_to = tpos.distance(*pos);
                    if dist_to < dist && dist_to > turret.min_distance {
                        dist = dist_to;
                        closest = pos;

                        // TODO Make the projectile hit the center of the target instead of the zero
                        // point
                        //closest.x += ubb.width() / 2.0;
                        //closest.y += ubb.height() / 2.0;
                    }
                }
            }

            let mut variation = 0.0;
            if turret.strength_variation > 0.0 {
                let between = Range::new(-turret.strength_variation, turret.strength_variation);
                variation = 1.0 + between.ind_sample(&mut rand::thread_rng());
            }

            let time = turret.flight_time;
            let mut vx = ((closest.x - tpos.x) / time) * variation;
            let mut vy = ((closest.y + 0.5 * -grav * time * time - tpos.y) / time) * variation;

            if (vx * vx + vy * vy).sqrt() < turret.max_strength {
                // Shoot the turret
                let projectile = entities.create();
                updater.insert(projectile, Projectile);
                updater.insert(projectile, WorldPosition(Point::new(tpos.x, tpos.y)));
                updater.insert(projectile, Velocity::new(vx, vy));
                updater.insert(projectile, *bb);
                updater.insert(projectile, *dmg);
                let entity: Option<&MaskId> = mask.get(e);
                if let Some(mask_e) = entity {
                    updater.insert(projectile, *mask_e);
                }
                let entity: Option<&ProjectileSprite> = sprite.get(e);
                if let Some(sprite_e) = entity {
                    updater.insert(projectile, sprite_e.0);
                }
                let entity: Option<&Arrow> = arrow.get(e);
                if let Some(arrow_e) = entity {
                    updater.insert(projectile, *arrow_e);
                }
                let entity: Option<&Line> = line.get(e);
                if let Some(line_e) = entity {
                    updater.insert(projectile, *line_e);
                }
                let entity: Option<&IgnoreCollision> = ignore.get(e);
                if let Some(ignore_e) = entity {
                    updater.insert(projectile, *ignore_e);
                }

                turret.delay_left = turret.delay;
            }
        }
    }
}

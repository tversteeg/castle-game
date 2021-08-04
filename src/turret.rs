use cgmath::MetricSpace;
use rand::distributions::{Distribution, Uniform};
use specs::prelude::*;
use specs_derive::Component;

use super::*;

#[derive(Component, Debug)]
pub struct Turret {
    pub delay: f64,
    pub min_distance: f64,
    pub max_strength: f64,
    pub flight_time: f64,
    pub strength_variation: f64,

    pub delay_left: f64,
}

impl Default for Turret {
    fn default() -> Self {
        Turret {
            delay: 5.0,
            min_distance: 20.0,
            max_strength: 210.0,
            flight_time: 3.0,
            strength_variation: 0.1,

            delay_left: 0.0,
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
pub struct TurretOffset(pub (f64, f64));

#[derive(SystemData)]
pub struct TurretUnitSystemData<'a> {
    turret: ReadStorage<'a, Turret>,
    wpos: ReadStorage<'a, WorldPosition>,
    offset: ReadStorage<'a, TurretOffset>,
    state: WriteStorage<'a, UnitState>,
    pos: WriteStorage<'a, Point>,
}

pub struct TurretUnitSystem;
impl<'a> System<'a> for TurretUnitSystem {
    type SystemData = TurretUnitSystemData<'a>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        for (turret, wpos, offset, state, pos) in (
            &system_data.turret,
            &system_data.wpos,
            &system_data.offset,
            &mut system_data.state,
            &mut system_data.pos,
        )
            .join()
        {
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

#[derive(SystemData)]
pub struct TurretSystemData<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    grav: Read<'a, Gravity>,
    ally: ReadStorage<'a, Ally>,
    enemy: ReadStorage<'a, Enemy>,
    pos: ReadStorage<'a, Point>,
    wpos: ReadStorage<'a, WorldPosition>,
    sprite: ReadStorage<'a, ProjectileSprite>,
    arrow: ReadStorage<'a, Arrow>,
    line: ReadStorage<'a, Line>,
    mask: ReadStorage<'a, MaskId>,
    ignore: ReadStorage<'a, IgnoreCollision>,
    bb: ReadStorage<'a, ProjectileBoundingBox>,
    ubb: ReadStorage<'a, BoundingBox>,
    dmg: ReadStorage<'a, Damage>,
    walk: ReadStorage<'a, Walk>,
    state: ReadStorage<'a, UnitState>,
    turret: WriteStorage<'a, Turret>,
    updater: Read<'a, LazyUpdate>,
}

pub struct TurretSystem;
impl<'a> System<'a> for TurretSystem {
    type SystemData = TurretSystemData<'a>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        let dt = system_data.dt.to_seconds();
        let grav = system_data.grav.0;

        for (e, tpos, bb, dmg, turret) in (
            &*system_data.entities,
            &system_data.pos,
            &system_data.bb,
            &system_data.dmg,
            &mut system_data.turret,
        )
            .join()
        {
            turret.delay_left -= dt;
            if turret.delay_left > 0.0 {
                continue;
            }

            // Find the nearest ally to shoot
            let mut closest = Point::new(-1000.0, -1000.0);
            let mut dist = tpos.distance(*closest);

            let is_ally: Option<&Ally> = system_data.ally.get(e);
            if is_ally.is_some() {
                for (epos, _, walk, ubb, state) in (
                    &system_data.wpos,
                    &system_data.enemy,
                    &system_data.walk,
                    &system_data.ubb,
                    &system_data.state,
                )
                    .join()
                {
                    let mut pos = epos.0;
                    pos.x += ubb.width() / 2.0;
                    pos.y += ubb.height() / 2.0;

                    if *state == UnitState::Walk {
                        pos.x += walk.speed * turret.flight_time;
                    }

                    let dist_to = tpos.distance(*pos);
                    if dist_to < dist && dist_to > turret.min_distance {
                        dist = dist_to;
                        closest = pos;
                    }
                }
            } else {
                for (apos, _, walk, ubb, state) in (
                    &system_data.wpos,
                    &system_data.ally,
                    &system_data.walk,
                    &system_data.ubb,
                    &system_data.state,
                )
                    .join()
                {
                    let mut pos = apos.0;
                    pos.x += ubb.width() / 2.0;
                    pos.y += ubb.height() / 2.0;

                    if *state == UnitState::Walk {
                        pos.x += walk.speed * turret.flight_time;
                    }

                    let dist_to = tpos.distance(*pos);
                    if dist_to < dist && dist_to > turret.min_distance {
                        dist = dist_to;
                        closest = pos;
                    }
                }
            }

            let variation = if turret.strength_variation > 0.0 {
                let between = if closest.x > tpos.x {
                    Uniform::new(0.0, turret.strength_variation)
                } else {
                    Uniform::new(-turret.strength_variation, 0.0)
                };

                between.sample(&mut rand::thread_rng()) * dist
            } else {
                1.0
            };

            let time = turret.flight_time;
            let vx = (closest.x - tpos.x + variation) / time;
            let vy = (closest.y + 0.5 * -grav * time * time - tpos.y) / time;

            if (vx * vx + vy * vy).sqrt() < turret.max_strength {
                // Shoot the turret
                let projectile = system_data.entities.create();
                system_data.updater.insert(projectile, Projectile);
                system_data
                    .updater
                    .insert(projectile, WorldPosition(Point::new(tpos.x, tpos.y)));
                system_data
                    .updater
                    .insert(projectile, Velocity::new(vx, vy));
                system_data.updater.insert(projectile, *bb);
                system_data.updater.insert(projectile, *dmg);
                let entity: Option<&MaskId> = system_data.mask.get(e);
                if let Some(mask_e) = entity {
                    system_data.updater.insert(projectile, *mask_e);
                }
                let entity: Option<&ProjectileSprite> = system_data.sprite.get(e);
                if let Some(sprite_e) = entity {
                    system_data.updater.insert(projectile, sprite_e.0);
                }
                let entity: Option<&Arrow> = system_data.arrow.get(e);
                if let Some(arrow_e) = entity {
                    system_data.updater.insert(projectile, *arrow_e);
                }
                let entity: Option<&Line> = system_data.line.get(e);
                if let Some(line_e) = entity {
                    system_data.updater.insert(projectile, *line_e);
                }
                let entity: Option<&IgnoreCollision> = system_data.ignore.get(e);
                if let Some(ignore_e) = entity {
                    system_data.updater.insert(projectile, *ignore_e);
                }

                turret.delay_left = turret.delay;
            }
        }
    }
}

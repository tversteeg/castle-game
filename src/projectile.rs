use crate::audio::Audio;
use collision::Discrete;
use rand::{
    self,
    distributions::{Distribution, Uniform},
};
use specs::prelude::*;
use specs_derive::Component;

use super::*;

const BLOOD_COLOR: u32 = 0xAC_32_33;

#[derive(Component, Debug, Copy, Clone, PartialEq, Eq)]
pub enum IgnoreCollision {
    Enemy,
    Ally,
}

#[derive(Component, Debug, Copy, Clone)]
pub struct Projectile;

#[derive(Component, Debug, Copy, Clone)]
pub struct ProjectileSprite(pub Sprite);

#[derive(Component, Debug, Copy, Clone)]
pub struct ProjectileBoundingBox(pub BoundingBox);

#[derive(Component, Debug, Copy, Clone)]
pub struct Arrow(pub f64);

#[derive(Component, Debug, Copy, Clone)]
pub struct Damage(pub f64);

pub struct ArrowSystem;
impl<'a> System<'a> for ArrowSystem {
    type SystemData = (
        ReadStorage<'a, WorldPosition>,
        ReadStorage<'a, Velocity>,
        WriteStorage<'a, Arrow>,
        WriteStorage<'a, Line>,
    );

    fn run(&mut self, (pos, vel, mut arrow, mut line): Self::SystemData) {
        for (pos, vel, arrow, line) in (&pos, &vel, &mut arrow, &mut line).join() {
            let rot = (vel.y as f64).atan2(vel.x as f64);

            line.p1.x = pos.0.x as usize;
            line.p1.y = pos.0.y as usize;

            line.p2.x = (pos.0.x - (rot.cos() * arrow.0)) as usize;
            line.p2.y = (pos.0.y - (rot.sin() * arrow.0)) as usize;
        }
    }
}

#[derive(SystemData)]
pub struct ProjectileSystemData<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    grav: Read<'a, Gravity>,
    terrain: Read<'a, Terrain>,
    audio: Read<'a, Audio>,
    proj: ReadStorage<'a, Projectile>,
    mask: ReadStorage<'a, MaskId>,
    line: WriteStorage<'a, Line>,
    vel: WriteStorage<'a, Velocity>,
    pos: WriteStorage<'a, WorldPosition>,
    updater: Read<'a, LazyUpdate>,
}

pub struct ProjectileSystem;
impl<'a> System<'a> for ProjectileSystem {
    type SystemData = ProjectileSystemData<'a>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        let grav = system_data.grav.0;
        let dt = system_data.dt.to_seconds();

        for (entity, _, vel, pos) in (
            &*system_data.entities,
            &system_data.proj,
            &mut system_data.vel,
            &mut system_data.pos,
        )
            .join()
        {
            let next: Point = Point::new(pos.0.x + vel.x * dt, pos.0.y + vel.y * dt);

            match system_data
                .terrain
                .line_collides(pos.0.as_i32(), next.as_i32())
            {
                Some(point) => {
                    if point.0 < 0 || point.1 < 0 {
                        let _ = system_data.entities.delete(entity);
                        continue;
                    }

                    if let Some(mask) = system_data.mask.get(entity) {
                        // Create a crater if there is a mask for it
                        system_data.updater.insert(
                            system_data.entities.create(),
                            TerrainMask::new(mask.id, point, mask.size),
                        );

                        // Play a sound
                        system_data.audio.play_heavy_projectile();
                    }

                    if let Some(line) = system_data.line.get(entity) {
                        // Keep drawing the line if there is one, this makes the arrows stay stuck
                        // in the ground
                        let mut line_copy = *line;
                        line_copy.p1.x = point.0 as usize;
                        line_copy.p1.y = point.1 as usize;

                        // Calculate the end point but be wary of integer overflows
                        let dx = point.0 - line.p1.x as i32;
                        let dy = point.1 - line.p1.y as i32;
                        if line.p2.x as i32 + dx >= 0 && line.p2.y as i32 + dy >= 0 {
                            line_copy.p2.x = (line_copy.p2.x as i32 + dx) as usize;
                            line_copy.p2.y = (line_copy.p2.y as i32 + dy) as usize;

                            system_data
                                .updater
                                .insert(system_data.entities.create(), line_copy);
                        }

                        // Play a sound
                        system_data.audio.play_light_projectile();
                    }

                    let _ = system_data.entities.delete(entity);
                }
                None => {
                    pos.0 = next;
                    vel.y += grav * dt;
                }
            }
        }
    }
}

#[derive(SystemData)]
pub struct ProjectileCollisionSystemData<'a> {
    entities: Entities<'a>,
    audio: Read<'a, Audio>,
    updater: Read<'a, LazyUpdate>,
    proj: ReadStorage<'a, Projectile>,
    pos: ReadStorage<'a, WorldPosition>,
    proj_bb: ReadStorage<'a, ProjectileBoundingBox>,
    bb: ReadStorage<'a, BoundingBox>,
    dmg: ReadStorage<'a, Damage>,
    ignore: ReadStorage<'a, IgnoreCollision>,
    ally: ReadStorage<'a, Ally>,
    enemy: ReadStorage<'a, Enemy>,
    health: WriteStorage<'a, Health>,
}

pub struct ProjectileCollisionSystem;
impl<'a> System<'a> for ProjectileCollisionSystem {
    type SystemData = ProjectileCollisionSystemData<'a>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        for (proj, _, proj_pos, proj_bb, proj_dmg) in (
            &*system_data.entities,
            &system_data.proj,
            &system_data.pos,
            &system_data.proj_bb,
            &system_data.dmg,
        )
            .join()
        {
            let proj_aabb = proj_bb.0 + *proj_pos.0;
            for (target, target_pos, target_bb, target_health) in (
                &*system_data.entities,
                &system_data.pos,
                &system_data.bb,
                &mut system_data.health,
            )
                .join()
            {
                let ignore_e: Option<&IgnoreCollision> = system_data.ignore.get(proj);
                if let Some(ignore) = ignore_e {
                    if *ignore == IgnoreCollision::Ally {
                        let is_ally: Option<&Ally> = system_data.ally.get(target);
                        if is_ally.is_some() {
                            continue;
                        }
                    }
                    if *ignore == IgnoreCollision::Enemy {
                        let is_enemy: Option<&Enemy> = system_data.enemy.get(target);
                        if is_enemy.is_some() {
                            continue;
                        }
                    }
                }

                // When there is a collision with a unit
                let target_aabb = *target_bb + *target_pos.0;
                if proj_aabb.intersects(&*target_aabb) {
                    if reduce_unit_health(&system_data.entities, target, target_health, proj_dmg.0)
                    {
                        // The ally died
                        system_data.updater.insert(
                            system_data.entities.create(),
                            FloatingText {
                                text: "x".to_string(),
                                pos: target_pos.0,
                                time_alive: 2.0,
                            },
                        );
                    }

                    let _ = system_data.entities.delete(proj);
                    let between = Uniform::new(-20.0, 20.0);
                    let mut rng = rand::thread_rng();

                    for _ in 0..4 {
                        let blood = system_data.entities.create();
                        system_data
                            .updater
                            .insert(blood, PixelParticle::new(BLOOD_COLOR, 10.0));
                        system_data.updater.insert(blood, *target_pos);
                        system_data.updater.insert(
                            blood,
                            Velocity::new(between.sample(&mut rng), between.sample(&mut rng)),
                        );
                    }

                    // Play a sound
                    system_data.audio.play_unit_hit();
                }
            }
        }
    }
}

pub struct ProjectileRemovalFromMaskSystem;
impl<'a> System<'a> for ProjectileRemovalFromMaskSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, TerrainMask>,
        ReadStorage<'a, Line>,
    );

    fn run(&mut self, (entities, mask, line): Self::SystemData) {
        for mask in mask.join() {
            let sx = (mask.size.0 / 2) as usize;
            let sy = (mask.size.1 / 2) as usize;

            for (entity, line) in (&*entities, &line).join() {
                // Check if the line's start point is inside the mask and remove it if that's the case
                let dx = (mask.pos.0 - line.p1.x as i32).abs() as usize;
                let dy = (mask.pos.1 - line.p1.y as i32).abs() as usize;
                if dx <= sx && dy <= sy {
                    let _ = entities.delete(entity);
                }
            }
        }
    }
}

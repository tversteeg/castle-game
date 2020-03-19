use specs::*;
use specs_derive::Component;
use std::time::Duration;

use super::*;

#[derive(Component, Debug, Copy, Clone)]
pub struct Velocity {
    pub x: f64,
    pub y: f64,
}

impl Velocity {
    pub fn new(x: f64, y: f64) -> Self {
        Velocity { x, y }
    }
}

#[derive(Default)]
pub struct DeltaTime(pub Duration);

impl DeltaTime {
    pub fn new(time: f64) -> Self {
        DeltaTime(Duration::from_millis((time * 1000.0) as u64))
    }

    pub fn to_seconds(&self) -> f64 {
        self.0.as_secs() as f64 + self.0.subsec_nanos() as f64 * 1e-9
    }
}

#[derive(Default)]
pub struct Gravity(pub f64);

pub struct ParticleSystem;
impl<'a> System<'a> for ParticleSystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, DeltaTime>,
        Read<'a, Gravity>,
        Write<'a, Terrain>,
        WriteStorage<'a, WorldPosition>,
        WriteStorage<'a, Velocity>,
        WriteStorage<'a, PixelParticle>,
    );

    fn run(
        &mut self,
        (entities, dt, grav, mut terrain, mut pos, mut vel, mut par): Self::SystemData,
    ) {
        let grav = grav.0;
        let dt = dt.to_seconds();

        for (entity, pos, vel, par) in (&*entities, &mut pos, &mut vel, &mut par).join() {
            pos.0.x += vel.x * dt;
            pos.0.y += vel.y * dt;
            vel.y += grav * dt;

            let old_pos = par.pos;
            match terrain.line_collides(pos.0.as_i32(), (old_pos.x as i32, old_pos.y as i32)) {
                Some(point) => {
                    terrain.draw_pixel((point.0 as usize, point.1 as usize), par.color);
                    let _ = entities.delete(entity);
                }
                None => {
                    par.pos = pos.0.as_usize();
                    par.life -= dt;
                    if par.life < 0.0 {
                        let _ = entities.delete(entity);
                    }
                }
            }
        }
    }
}

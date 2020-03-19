use specs::prelude::*;
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

#[derive(SystemData)]
pub struct ParticleSystemData<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    grav: Read<'a, Gravity>,
    terrain: Write<'a, Terrain>,
    pos: WriteStorage<'a, WorldPosition>,
    vel: WriteStorage<'a, Velocity>,
    par: WriteStorage<'a, PixelParticle>,
}

pub struct ParticleSystem;
impl<'a> System<'a> for ParticleSystem {
    type SystemData = ParticleSystemData<'a>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        let grav = system_data.grav.0;
        let dt = system_data.dt.to_seconds();

        for (entity, pos, vel, par) in (
            &*system_data.entities,
            &mut system_data.pos,
            &mut system_data.vel,
            &mut system_data.par,
        )
            .join()
        {
            pos.0.x += vel.x * dt;
            pos.0.y += vel.y * dt;
            vel.y += grav * dt;

            let old_pos = par.pos;
            match system_data
                .terrain
                .line_collides(pos.0.as_i32(), (old_pos.x as i32, old_pos.y as i32))
            {
                Some(point) => {
                    system_data
                        .terrain
                        .draw_pixel((point.0 as usize, point.1 as usize), par.color);
                    let _ = system_data.entities.delete(entity);
                }
                None => {
                    par.pos = pos.0.as_usize();
                    par.life -= dt;
                    if par.life < 0.0 {
                        let _ = system_data.entities.delete(entity);
                    }
                }
            }
        }
    }
}

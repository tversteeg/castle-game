use collision::Discrete;
use specs::prelude::*;
use specs_derive::Component;

use super::*;

const BLOOD_COLOR: u32 = 0xAC_32_33;

#[derive(Component, Debug, Copy, Clone)]
pub struct Destination(pub f64);

#[derive(Component, Debug)]
pub struct Ally;

#[derive(Component, Debug)]
pub struct Enemy;

#[derive(Component, Debug)]
pub struct Melee {
    dmg: f64,
    hitrate: f64,

    cooldown: f64,
}

impl Melee {
    pub fn new(dmg: f64, hitrate: f64) -> Self {
        Melee {
            dmg,
            hitrate,

            cooldown: 0.0,
        }
    }
}

#[derive(SystemData)]
pub struct MeleeSystemData<'a> {
    entities: Entities<'a>,
    dt: Read<'a, DeltaTime>,
    ally: ReadStorage<'a, Ally>,
    enemy: ReadStorage<'a, Enemy>,
    pos: ReadStorage<'a, WorldPosition>,
    bb: ReadStorage<'a, BoundingBox>,
    state: ReadStorage<'a, UnitState>,
    melee: WriteStorage<'a, Melee>,
    health: WriteStorage<'a, Health>,
    updater: Read<'a, LazyUpdate>,
}

pub struct MeleeSystem;
impl<'a> System<'a> for MeleeSystem {
    type SystemData = MeleeSystemData<'a>;

    fn run(&mut self, mut system_data: Self::SystemData) {
        let dt = system_data.dt.to_seconds();

        for (a, _, a_pos, a_bb, a_state) in (
            &*system_data.entities,
            &system_data.ally,
            &system_data.pos,
            &system_data.bb,
            &system_data.state,
        )
            .join()
        {
            // Only fight between units with the melee state
            if *a_state != UnitState::Melee {
                continue;
            }

            let a_aabb = *a_bb + *a_pos.0;
            for (e, _, e_pos, e_bb) in (
                &*system_data.entities,
                &system_data.enemy,
                &system_data.pos,
                &system_data.bb,
            )
                .join()
            {
                // Only fight between units with the melee state
                if *a_state != UnitState::Melee {
                    continue;
                }

                let e_aabb = *e_bb + *e_pos.0;
                if a_aabb.intersects(&*e_aabb) {
                    {
                        let a_melee: Option<&mut Melee> = system_data.melee.get_mut(a);
                        if let Some(melee) = a_melee {
                            melee.cooldown -= dt;
                            if melee.cooldown <= 0.0 {
                                if reduce_unit_health(
                                    &system_data.entities,
                                    e,
                                    system_data.health.get_mut(e).unwrap(),
                                    melee.dmg,
                                ) {
                                    // The enemy died
                                    system_data.updater.insert(
                                        system_data.entities.create(),
                                        FloatingText {
                                            text: "x".to_string(),
                                            pos: e_pos.0,
                                            time_alive: 2.0,
                                        },
                                    );
                                }

                                melee.cooldown = melee.hitrate;

                                let blood = system_data.entities.create();
                                system_data
                                    .updater
                                    .insert(blood, PixelParticle::new(BLOOD_COLOR, 10.0));
                                system_data.updater.insert(blood, *e_pos);
                                system_data
                                    .updater
                                    .insert(blood, Velocity::new(-10.0, -10.0));
                            }
                        }
                    }
                    {
                        let e_melee: Option<&mut Melee> = system_data.melee.get_mut(e);
                        if let Some(melee) = e_melee {
                            melee.cooldown -= dt;
                            if melee.cooldown <= 0.0 {
                                if reduce_unit_health(
                                    &system_data.entities,
                                    a,
                                    system_data.health.get_mut(a).unwrap(),
                                    melee.dmg,
                                ) {
                                    // The ally died
                                    system_data.updater.insert(
                                        system_data.entities.create(),
                                        FloatingText {
                                            text: "x".to_string(),
                                            pos: a_pos.0,
                                            time_alive: 2.0,
                                        },
                                    );
                                }

                                melee.cooldown = melee.hitrate;

                                let blood = system_data.entities.create();
                                system_data
                                    .updater
                                    .insert(blood, PixelParticle::new(BLOOD_COLOR, 10.0));
                                system_data.updater.insert(blood, *a_pos);
                                system_data
                                    .updater
                                    .insert(blood, Velocity::new(-10.0, -10.0));
                            }
                        }
                    }
                }
            }
        }
    }
}

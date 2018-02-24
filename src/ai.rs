use specs::*;
use collision::Discrete;

use super::*;

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

    cooldown: f64
}

impl Melee {
    pub fn new(dmg: f64, hitrate: f64) -> Self {
        Melee {
            dmg, hitrate,

            cooldown: 0.0
        }
    }
}

pub struct MeleeSystem;
impl<'a> System<'a> for MeleeSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       ReadStorage<'a, Ally>,
                       ReadStorage<'a, Enemy>,
                       ReadStorage<'a, WorldPosition>,
                       ReadStorage<'a, BoundingBox>,
                       WriteStorage<'a, UnitState>,
                       WriteStorage<'a, Melee>,
                       WriteStorage<'a, Health>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, ally, enemy, pos, bb, mut state, mut melee, mut health, updater): Self::SystemData) {
        let dt = dt.to_seconds();

        for (a, _, a_pos, a_bb) in (&*entities, &ally, &pos, &bb).join() {
            let a_aabb = *a_bb + *a_pos.0;
            for (e, _, e_pos, e_bb) in (&*entities, &enemy, &pos, &bb).join() {
                let e_aabb = *e_bb + *e_pos.0;
                if a_aabb.intersects(&*e_aabb) {
                    {
                        let a_state: &mut UnitState = state.get_mut(a).unwrap();
                        *a_state = UnitState::Melee;

                        let a_melee: Option<&mut Melee> = melee.get_mut(a);
                        if let Some(melee) = a_melee {
                            melee.cooldown -= dt;
                            if melee.cooldown <= 0.0 {
                                if reduce_unit_health(&entities, &e, health.get_mut(e).unwrap(), melee.dmg) {
                                    *a_state = UnitState::Walk;
                                }

                                melee.cooldown = melee.hitrate;

                                let blood = entities.create();
                                updater.insert(blood, PixelParticle::new(0xFF0000, 10.0));
                                updater.insert(blood, *e_pos);
                                updater.insert(blood, Velocity::new(-10.0, -10.0));
                            }
                        }
                    }
                    {
                        let e_state: &mut UnitState = state.get_mut(e).unwrap();
                        *e_state = UnitState::Melee;

                        let e_melee: Option<&mut Melee> = melee.get_mut(e);
                        if let Some(melee) = e_melee {
                            melee.cooldown -= dt;
                            if melee.cooldown <= 0.0 {
                                if reduce_unit_health(&entities, &a, health.get_mut(a).unwrap(), melee.dmg) {
                                    *e_state = UnitState::Walk;
                                }

                                melee.cooldown = melee.hitrate;

                                let blood = entities.create();
                                updater.insert(blood, PixelParticle::new(0xFF0000, 10.0));
                                updater.insert(blood, *a_pos);
                                updater.insert(blood, Velocity::new(-10.0, -10.0));
                            }
                        }
                    }
                }
            }
        }
    }
}

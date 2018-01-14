use specs::*;

use physics::*;
use draw::*;
use terrain::*;

pub struct ProjectileSystem;

impl<'a> System<'a> for ProjectileSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       Fetch<'a, Terrain>,
                       WriteStorage<'a, Velocity>,
                       WriteStorage<'a, Position>,
                       ReadStorage<'a, MaskId>,
                       Fetch<'a, LazyUpdate>);

    fn run(&mut self, (entities, dt, grav, terrain, mut vel, mut pos, mask, updater): Self::SystemData) {
        let grav = grav.0;
        let dt = dt.to_seconds();

        for (entity, vel, pos, mask) in (&*entities, &mut vel, &mut pos, &mask).join() {
            let next = Position::new(pos.x + vel.x * dt, pos.y + vel.y * dt);

            match terrain.line_collides(pos.as_i32(), next.as_i32()) {
                Some(point) => {
                    let _ = entities.delete(entity);

                    let crater = entities.create();
                    updater.insert(crater, TerrainMask::new(mask.0, point));
                },
                None => {
                    *pos = next;
                    vel.y += grav * dt;
                }
            }
        }
    }
}

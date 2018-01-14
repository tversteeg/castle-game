use specs::*;

use physics::*;
use terrain::*;

#[derive(Component)]
pub struct Health(i32);

pub struct UnitSystem;

impl<'a> System<'a> for UnitSystem {
    type SystemData = (Entities<'a>,
                       Fetch<'a, DeltaTime>,
                       Fetch<'a, Gravity>,
                       Fetch<'a, Terrain>,
                       WriteStorage<'a, Health>);

    fn run(&mut self, (entities, dt, grav, terrain, mut health): Self::SystemData) {

    }
}

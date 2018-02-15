use specs::*;

use ::*;

pub fn place_turrets(world: &mut World, level: u8) {
    let (projectile1, bighole1, ally_melee1, ally_archer1, enemy_soldier1) = {
        let images = &*world.read_resource::<Images>();

        (*images.0.get("projectile1").unwrap(),
         *images.0.get("bighole1").unwrap(),
         *images.0.get("ally-melee1").unwrap(),
         *images.0.get("ally-archer1").unwrap(),
         *images.0.get("enemy-melee1").unwrap())
    };

    match level {
        1 => {
            world.create_entity()
                .with(Turret::default())
                .with(Point::new(630.0, 190.0))
                .with(Enemy)
                .with(Sprite::new(projectile1))
                .with(MaskId(bighole1))
                .with(bb(p(0.0, 0.0), p(5.0, 5.0)))
                .with(Damage(30.0))
                .build();

            world.create_entity()
                .with(Turret::new(2.0, 200.0, 5.0, 3.0))
                .with(Point::new(615.0, 205.0))
                .with(Enemy)
                .with(Arrow(3.0))
                .with(Line::new(0x000000))
                .with(MaskId(bighole1))
                .with(bb(p(0.0, 0.0), p(1.0, 1.0)))
                .with(Damage(10.0))
                .build();

            world.create_entity()
                .with(Sprite::new(ally_melee1))
                .with(Point::new(12.0, 200.0))
                .with(Walk::new(bb(p(1.0, 5.0), p(4.0, 10.0)), 10.0))
                .with(bb(p(0.0, 0.0), p(10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(100.0))
                .with(Melee::new(10.0, 1.0))
                .with(UnitState::Walk)
                .with(Ally)
                .build();

            world.create_entity()
                .with(Sprite::new(ally_melee1))
                .with(Point::new(5.0, 200.0))
                .with(Walk::new(bb(p(1.0, 5.0), p(4.0, 10.0)), 10.0))
                .with(bb(p(0.0, 0.0), p(10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(100.0))
                .with(Melee::new(10.0, 1.0))
                .with(UnitState::Walk)
                .with(Ally)
                .build();

            world.create_entity()
                .with(Sprite::new(enemy_soldier1))
                .with(Point::new(570.0, 200.0))
                .with(Walk::new(bb(p(2.0, 5.0), p(5.0, 10.0)), 10.0))
                .with(bb(p(0.0, 0.0), p(10.0, 10.0)))
                .with(Destination(10.0))
                .with(Health(100.0))
                .with(Melee::new(10.0, 1.0))
                .with(UnitState::Walk)
                .with(Enemy)
                .build();
        },
        _ => ()
    }
}

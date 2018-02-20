use specs::*;

use ::*;

pub fn buy_archer(world: &mut World) {
    let archer_sprite = {
        let images = &*world.read_resource::<Images>();

        *images.0.get("ally-archer1").unwrap()
    };

    world.create_entity()
        .with(Ally)
        .with(Sprite::new(archer_sprite))
        .with(WorldPosition(Point::new(1.0, 230.0)))
        .with(Walk::new(BoundingBox::new(Point::new(1.0, 5.0), Point::new(4.0, 10.0)), 10.0))
        .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
        .with(Destination(630.0))
        .with(Health(20.0))
        .with(Melee::new(5.0, 1.0))
        .with(Turret::new(3.0, 150.0, 5.0, 1.0))
        .with(TurretOffset((2.0, 2.0)))
        .with(Point::new(0.0, 0.0))
        .with(Arrow(3.0))
        .with(Line::new(0x4C2D24))
        .with(Damage(5.0))
        .with(ProjectileBoundingBox(BoundingBox::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0))))
        .with(IgnoreCollision::Ally)
        .with(UnitState::Walk)
        .build();
}

pub fn place_turrets(world: &mut World, level: u8) {
    let (projectile1, bighole1, ally_melee1, enemy_soldier1, enemy_archer1) = {
        let images = &*world.read_resource::<Images>();

        (*images.0.get("projectile1").unwrap(),
         *images.0.get("bighole1").unwrap(),
         *images.0.get("ally-melee1").unwrap(),
         *images.0.get("enemy-melee1").unwrap(),
         *images.0.get("enemy-archer1").unwrap())
    };

    match level {
        1 => {
            world.create_entity()
                .with(Enemy)
                .with(Turret::default())
                .with(Point::new(630.0, 190.0))
                .with(ProjectileSprite(Sprite::new(projectile1)))
                .with(MaskId(bighole1))
                .with(ProjectileBoundingBox(BoundingBox::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0))))
                .with(Damage(30.0))
                .build();

            world.create_entity()
                .with(Enemy)
                .with(Turret::new(1.0, 190.0, 5.0, 3.0))
                .with(Point::new(615.0, 205.0))
                .with(Arrow(5.0))
                .with(Line::new(0x4C2D24))
                .with(ProjectileBoundingBox(BoundingBox::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0))))
                .with(Damage(10.0))
                .build();

            world.create_entity()
                .with(Ally)
                .with(Sprite::new(ally_melee1))
                .with(WorldPosition(Point::new(2.0, 200.0)))
                .with(Walk::new(BoundingBox::new(Point::new(1.0, 5.0), Point::new(4.0, 10.0)), 10.0))
                .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(50.0))
                .with(Melee::new(10.0, 1.0))
                .with(UnitState::Walk)
                .build();

            world.create_entity()
                .with(Ally)
                .with(Sprite::new(ally_melee1))
                .with(WorldPosition(Point::new(70.0, 200.0)))
                .with(Walk::new(BoundingBox::new(Point::new(1.0, 5.0), Point::new(4.0, 10.0)), 10.0))
                .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(50.0))
                .with(Melee::new(10.0, 1.0))
                .with(UnitState::Walk)
                .build();

            for i in 0..10 {
                world.create_entity()
                    .with(Enemy)
                    .with(Sprite::new(enemy_soldier1))
                    .with(WorldPosition(Point::new(570.0 - 20.0 * i as f64, 200.0)))
                    .with(Walk::new(BoundingBox::new(Point::new(2.0, 5.0), Point::new(5.0, 10.0)), 10.0))
                    .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
                    .with(Destination(10.0))
                    .with(Health(50.0))
                    .with(Melee::new(10.0, 1.0))
                    .with(UnitState::Walk)
                    .build();
            }

            for i in 0..20 {
                world.create_entity()
                    .with(Enemy)
                    .with(Sprite::new(enemy_archer1))
                    .with(WorldPosition(Point::new(580.0 - 20.0 * i as f64, 200.0)))
                    .with(Walk::new(BoundingBox::new(Point::new(1.0, 5.0), Point::new(4.0, 10.0)), 10.0))
                    .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
                    .with(Destination(10.0))
                    .with(Health(20.0))
                    .with(Melee::new(5.0, 1.0))
                    .with(Turret::new(3.0, 150.0, 5.0, 1.0))
                    .with(TurretOffset((2.0, 2.0)))
                    .with(Point::new(0.0, 0.0))
                    .with(Arrow(3.0))
                    .with(Line::new(0x4C2D24))
                    .with(Damage(5.0))
                    .with(ProjectileBoundingBox(BoundingBox::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0))))
                    .with(IgnoreCollision::Enemy)
                    .with(UnitState::Walk)
                    .build();
            }
        },
        _ => ()
    }
}

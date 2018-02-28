use specs::*;
use cgmath::Point2;

use ::*;

pub fn buy_archer(world: &mut World) {
    let archer_sprite = {
        let images = &*world.read_resource::<Images>();

        *images.0.get("ally-archer1").unwrap()
    };

    let health = 20.0;

    world.create_entity()
        .with(Ally)
        .with(Sprite::new(archer_sprite))
        .with(WorldPosition(Point::new(1.0, 340.0)))
        .with(Walk::new(BoundingBox::new(Point::new(1.0, 5.0), Point::new(4.0, 10.0)), 20.0))
        .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
        .with(Destination(1280.0))
        .with(Health(20.0))
        .with(HealthBar {
            health,
            max_health: health,
            width: 5,
            pos: Point2::new(0, 0),
            offset: (1, -3)
        })
        .with(Melee::new(5.0, 1.0))
        .with(Turret {
            delay: 3.0,
            min_distance: 20.0,
            max_strength: 150.0,
            flight_time: 2.0,
            strength_variation: 0.1,
            ..Turret::default()
        })
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

pub fn buy_soldier(world: &mut World) {
    let soldier_sprite = {
        let images = &*world.read_resource::<Images>();

        *images.0.get("ally-melee1").unwrap()
    };

    let health = 50.0;

    world.create_entity()
        .with(Ally)
        .with(Sprite::new(soldier_sprite))
        .with(WorldPosition(Point::new(1.0, 340.0)))
        .with(Walk::new(BoundingBox::new(Point::new(1.0, 5.0), Point::new(4.0, 10.0)), 15.0))
        .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
        .with(Destination(1280.0))
        .with(Health(health))
        .with(HealthBar {
            health,
            max_health: health,
            width: 10,
            pos: Point2::new(0, 0),
            offset: (-2, -3)
        })
        .with(Melee::new(10.0, 1.0))
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
                .with(Turret {
                    delay: 3.0,
                    min_distance: 50.0,
                    max_strength: 310.0,
                    flight_time: 5.0,
                    strength_variation: 0.05,
                    ..Turret::default()
                })
                .with(Point::new(1270.0, 295.0))
                .with(ProjectileSprite(Sprite::new(projectile1)))
                .with(MaskId { id: bighole1, size: (5, 5) })
                .with(ProjectileBoundingBox(BoundingBox::new(Point::new(0.0, 0.0), Point::new(5.0, 5.0))))
                .with(Damage(30.0))
                .build();

            world.create_entity()
                .with(Enemy)
                .with(Turret {
                    delay: 1.0,
                    min_distance: 50.0,
                    max_strength: 290.0,
                    flight_time: 4.0,
                    strength_variation: 0.05,
                    ..Turret::default()
                })
                .with(Point::new(1255.0, 315.0))
                .with(Arrow(10.0))
                .with(Line::new(0x4C2D24))
                .with(ProjectileBoundingBox(BoundingBox::new(Point::new(0.0, 0.0), Point::new(1.0, 1.0))))
                .with(Damage(10.0))
                .build();

            for i in 0..10 {
                let health = 50.0;

                world.create_entity()
                    .with(Enemy)
                    .with(Sprite::new(enemy_soldier1))
                    .with(WorldPosition(Point::new(1130.0 - 20.0 * i as f64, 320.0)))
                    .with(Walk::new(BoundingBox::new(Point::new(2.0, 5.0), Point::new(5.0, 10.0)), 15.0))
                    .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
                    .with(Destination(10.0))
                    .with(Health(health))
                    .with(HealthBar {
                        health,
                        max_health: health,
                        width: 10,
                        pos: Point2::new(0, 0),
                        offset: (-2, -3)
                    })
                    .with(Melee::new(10.0, 1.0))
                    .with(UnitState::Walk)
                    .build();
            }

            for i in 0..40 {
                let health = 20.0;

                world.create_entity()
                    .with(Enemy)
                    .with(Sprite::new(enemy_archer1))
                    .with(WorldPosition(Point::new(1140.0 - 20.0 * i as f64, 320.0)))
                    .with(Walk::new(BoundingBox::new(Point::new(1.0, 5.0), Point::new(4.0, 10.0)), 20.0))
                    .with(BoundingBox::new(Point::new(0.0, 0.0), Point::new(10.0, 10.0)))
                    .with(Destination(10.0))
                    .with(Health(health))
                    .with(HealthBar {
                        health,
                        max_health: health,
                        width: 5,
                        pos: Point2::new(0, 0),
                        offset: (1, -3)
                    })
                    .with(Melee::new(5.0, 1.0))
                    .with(Turret {
                        delay: 3.0,
                        min_distance: 20.0,
                        max_strength: 150.0,
                        flight_time: 2.0,
                        strength_variation: 0.1,
                        ..Turret::default()
                    })
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

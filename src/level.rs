use specs::*;

use ::*;

pub fn place_turrets(world: &mut World, level: u8) {
    let (projectile1, bighole1, ally_soldier1, enemy_soldier1) = {
        let images = &*world.read_resource::<Images>();

        (*images.0.get("projectile1").unwrap(),
         *images.0.get("bighole1").unwrap(),
         *images.0.get("ally-soldier1").unwrap(),
         *images.0.get("enemy-soldier1").unwrap())
    };

    match level {
        1 => {
            world.create_entity()
                .with(Turret::default())
                .with(Position::new(630.0, 190.0))
                .with(Enemy)
                .with(Sprite::new(projectile1))
                .with(MaskId(bighole1))
                .with(BoundingBox(Rect::new(0.0, 0.0, 5.0, 5.0)))
                .with(Damage(30.0))
                .build();

            world.create_entity()
                .with(Turret::new(6.0, 230.0, 10.0, 4.0))
                .with(Position::new(610.0, 215.0))
                .with(Enemy)
                .with(Sprite::new(projectile1))
                .with(MaskId(bighole1))
                .with(BoundingBox(Rect::new(0.0, 0.0, 5.0, 5.0)))
                .with(Damage(30.0))
                .build();

            world.create_entity()
                .with(Sprite::new(ally_soldier1))
                .with(Position::new(12.0, 200.0))
                .with(Walk::new(Rect::new(1.0, 5.0, 3.0, 5.0), 10.0))
                .with(BoundingBox(Rect::new(0.0, 0.0, 10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(100.0))
                .with(Melee::new(10.0, 1.0))
                .with(Ally)
                .build();

            world.create_entity()
                .with(Sprite::new(ally_soldier1))
                .with(Position::new(5.0, 200.0))
                .with(Walk::new(Rect::new(1.0, 5.0, 3.0, 5.0), 10.0))
                .with(BoundingBox(Rect::new(0.0, 0.0, 10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(100.0))
                .with(Melee::new(10.0, 1.0))
                .with(Ally)
                .build();

            world.create_entity()
                .with(Sprite::new(enemy_soldier1))
                .with(Position::new(570.0, 200.0))
                .with(Walk::new(Rect::new(2.0, 5.0, 3.0, 5.0), 10.0))
                .with(BoundingBox(Rect::new(0.0, 0.0, 10.0, 10.0)))
                .with(Destination(10.0))
                .with(Health(100.0))
                .with(Melee::new(10.0, 1.0))
                .with(Enemy)
                .build();
        },
        _ => ()
    }
}

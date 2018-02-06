use specs::*;

use ::*;

pub fn place_turrets(world: &mut World, level: u8) {
    let (projectile1, bighole1, soldier) = {
        let images = &*world.read_resource::<Images>();

        (*images.0.get("projectile1").unwrap(),
         *images.0.get("bighole1").unwrap(),
         *images.0.get("soldier1").unwrap())
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
                .with(Sprite::new(soldier))
                .with(Position::new(12.0, 200.0))
                .with(Walk::new(Rect::new(1.0, 5.0, 3.0, 5.0), 10.0))
                .with(BoundingBox(Rect::new(0.0, 0.0, 10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(100.0))
                .with(Ally)
                .build();

            world.create_entity()
                .with(Sprite::new(soldier))
                .with(Position::new(5.0, 200.0))
                .with(Walk::new(Rect::new(1.0, 5.0, 3.0, 5.0), 10.0))
                .with(BoundingBox(Rect::new(0.0, 0.0, 10.0, 10.0)))
                .with(Destination(630.0))
                .with(Health(100.0))
                .with(Ally)
                .build();
        },
        _ => ()
    }
}

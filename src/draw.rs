use specs::*;
use sprite::Sprite;
use geom::Position;

pub struct Buffer {
    pub width: usize,
    pub height: usize,

    pub data: Vec<u32>
}

impl Buffer {
    pub fn new(size: (usize, usize)) -> Self {
        Buffer {
            width: size.0,
            height: size.1,

            data: vec![0; size.0 * size.1]
        }
    }
}

pub struct RenderSprite;

impl<'a> System<'a> for RenderSprite {
    type SystemData = (FetchMut<'a, Buffer>,
                       ReadStorage<'a, Position>,
                       ReadStorage<'a, Sprite>);

    fn run(&mut self, data: Self::SystemData) {
        let (mut buffer, pos, sprite) = data;

    }
}

pub fn register(world: &mut World) {
    world.register::<Sprite>();
}

pub fn load_sprites(world: &mut World) {
    world.create_entity().with(Sprite::new("assets/background.png"));
}

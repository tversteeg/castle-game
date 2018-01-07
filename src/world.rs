use specs::{World, System};
use draw;
use draw::Buffer;
use geom;
use window::GameWindow;

pub struct GameWorld {
    world: World,
    window: GameWindow
}

impl GameWorld {
    pub fn new(size: (usize, usize)) -> Self {
        let mut window = GameWindow::new(size);

        let mut world = World::new();

        draw::register(&mut world);
        geom::register(&mut world);

        draw::load_sprites(&mut world);

        world.add_resource(Buffer::new(size));
        
        GameWorld {
            world,
            window
        }
    }

    pub fn main_loop(&mut self) {
        let mut buffer = self.world.write_resource::<Buffer>();

        self.window.main_loop(&buffer.data);
    }
}

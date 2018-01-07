use minifb::*;

pub struct GameWindow {
    window: Window,
    size: (usize, usize)
}

impl GameWindow {
    pub fn new(size: (usize, usize)) -> Self {
        let options = WindowOptions {
            scale: Scale::X2,
            ..WindowOptions::default()
        };
        let mut window = Window::new("Castle Game - Press ESC to exit", size.0, size.1, options).expect("Unable to open window");

        GameWindow {
            window,
            size
        }
    }

    pub fn main_loop(&mut self, buffer: &Vec<u32>) {
        while self.window.is_open() && !self.window.is_key_down(Key::Escape) {
            self.window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
                let screen_pos = ((mouse.1 as usize) * self.size.0) + mouse.0 as usize;
            });

            self.window.get_keys().map(|keys| {
                for t in keys {
                    match t {
                        Key::W => println!("holding w!"),
                        Key::T => println!("holding t!"),
                        _ => (),
                    }
                }
            });

            self.window.update_with_buffer(buffer).unwrap();
        }
    }
}

extern crate minifb;
extern crate blit;
extern crate image;

use minifb::*;
use blit::*;
use image::GenericImage;

const WIDTH: usize = 640;
const HEIGHT: usize = 320;

const MASK_COLOR: u32 = 0xFFFF00FF;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let options = WindowOptions {
        scale: Scale::X2,
        ..WindowOptions::default()
    };
    let mut window = Window::new("Castle Game - Press ESC to exit", WIDTH, HEIGHT, options).expect("Unable to open window");

    let background_img = image::open("assets/background.png").unwrap();
    let background = background_img.as_rgb8().unwrap();
    background.blit_with_mask_color(&mut buffer, (WIDTH, HEIGHT), (0, 0), MASK_COLOR);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.get_mouse_pos(MouseMode::Discard).map(|mouse| {
            let screen_pos = ((mouse.1 as usize) * WIDTH) + mouse.0 as usize;

            if window.get_mouse_down(MouseButton::Left) {
                buffer[screen_pos] = 0x00ffffff;
            }

            if window.get_mouse_down(MouseButton::Right) {
                buffer[screen_pos] = 0;
            }
        });

        window.get_keys().map(|keys| {
            for t in keys {
                match t {
                    Key::W => println!("holding w!"),
                    Key::T => println!("holding t!"),
                    _ => (),
                }
            }
        });

        window.update_with_buffer(&buffer).unwrap();
    }
}

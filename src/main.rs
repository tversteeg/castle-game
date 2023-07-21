mod font;
mod game;
mod input;

use blit::prelude::Size;
use game::GameState;
use game_loop::winit::{dpi::LogicalSize, window::WindowBuilder};
use miette::{IntoDiagnostic, Result};
use pixels::{PixelsBuilder, SurfaceTexture};
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
use vek::Vec2;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoop,
};

const WIDTH: usize = 320;
const HEIGHT: usize = 180;
const FPS: usize = 60;

async fn run() -> Result<()> {
    // Construct the game
    let state = GameState::new();

    // Build the window builder with the event loop the user supplied
    let event_loop = EventLoop::new();
    let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
    let mut window_builder = WindowBuilder::new()
        .with_title("Castle Game")
        .with_inner_size(size)
        .with_min_inner_size(size);

    // Setup the WASM canvas if running on the browser
    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::WindowBuilderExtWebSys;

        window_builder = window_builder.with_canvas(Some(wasm::setup_canvas()));
    }

    let window = window_builder.build(&event_loop).unwrap();

    let pixels = {
        let surface_texture = SurfaceTexture::new(WIDTH as u32, HEIGHT as u32, &window);
        PixelsBuilder::new(WIDTH as u32, HEIGHT as u32, surface_texture)
            .clear_color(pixels::wgpu::Color {
                r: 0.796,
                g: 0.859,
                b: 0.988,
                a: 1.0,
            })
            .build_async()
            .await
    }
    .unwrap();

    // Open the window and run the event loop
    let mut buffer = vec![0u32; WIDTH * HEIGHT];

    game_loop::game_loop(
        event_loop,
        window,
        (state, pixels),
        FPS as u32,
        0.1,
        move |g| {
            // Update
            g.window
                .set_title(&format!("Update: {}", g.last_frame_time()));

            // Update the game
            g.game.0.update();
        },
        move |g| {
            buffer.fill(0);

            // Draw the game
            g.game.0.render(&mut buffer, Size::new(WIDTH, HEIGHT));

            // Blit draws the pixels in RGBA format, but the pixels crate expects BGRA, so convert it
            g.game
                .1
                .frame_mut()
                .chunks_exact_mut(4)
                .zip(buffer.iter())
                .for_each(|(target, source)| {
                    let source = source.to_ne_bytes();
                    target[0] = source[2];
                    target[1] = source[1];
                    target[2] = source[0];
                    target[3] = source[3];
                });

            // Render the pixel buffer
            if let Err(err) = g.game.1.render() {
                dbg!(err);
                // TODO: properly handle error
                g.exit();
            }
        },
        |g, event| {
            // Window events
            match event {
                // Handle key presses
                Event::WindowEvent {
                    event:
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    virtual_keycode,
                                    state,
                                    ..
                                },
                            ..
                        },
                    ..
                } => match virtual_keycode {
                    Some(VirtualKeyCode::Up | VirtualKeyCode::W) => {
                        g.game.0.input.up_pressed = state == &ElementState::Pressed
                    }
                    Some(VirtualKeyCode::Down | VirtualKeyCode::S) => {
                        g.game.0.input.down_pressed = state == &ElementState::Pressed
                    }
                    Some(VirtualKeyCode::Left | VirtualKeyCode::A) => {
                        g.game.0.input.left_pressed = state == &ElementState::Pressed
                    }
                    Some(VirtualKeyCode::Right | VirtualKeyCode::D) => {
                        g.game.0.input.right_pressed = state == &ElementState::Pressed
                    }
                    // Close the window when the <ESC> key is pressed
                    Some(VirtualKeyCode::Escape) => g.exit(),
                    _ => (),
                },

                // Handle mouse move
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    // Map raw window pixel to actual pixel
                    g.game.0.input.mouse_pos = g
                        .game
                        .1
                        .window_pos_to_pixel((position.x as f32, position.y as f32))
                        .map(|(x, y)| Vec2::new(x as f64, y as f64))
                        // We also map the mouse when it's outside of the bounds
                        .unwrap_or_else(|(x, y)| Vec2::new(x as f64, y as f64))
                }

                // Handle close event
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => g.exit(),

                // Resize the window
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    g.game
                        .1
                        .resize_surface(new_size.width, new_size.height)
                        .into_diagnostic()
                        .unwrap();
                }
                _ => (),
            }
        },
    );
}

fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(async { run().await.unwrap() });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let rt = Runtime::new().unwrap();
        rt.block_on(async { run().await.unwrap() });
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::JsCast;
    use web_sys::HtmlCanvasElement;

    /// Attach the winit window to a canvas.
    pub fn setup_canvas() -> HtmlCanvasElement {
        log::debug!("Binding window to HTML canvas");

        let window = web_sys::window().unwrap();

        let document = window.document().unwrap();
        let body = document.body().unwrap();
        body.style().set_css_text("text-align: center");

        let canvas = document
            .create_element("canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        canvas.set_id("canvas");
        body.append_child(&canvas).unwrap();
        canvas.style().set_css_text("display:block; margin: auto");

        let header = document.create_element("h2").unwrap();
        header.set_text_content(Some("Caste Game"));
        body.append_child(&header).unwrap();

        canvas
    }
}

mod camera;
#[cfg(feature = "debug")]
mod debug;
mod font;
mod game;
mod gen;
mod graphics;
mod object;
mod projectile;
mod random;
mod solid_shape;
mod sprite;
mod terrain;
mod timer;
mod unit;

use std::sync::OnceLock;

use assets_manager::{AssetReadGuard, Compound};
use font::Font;
use game::{GameState, Settings};
use miette::Result;
use pixel_game_lib::window::{KeyCode, WindowConfig};
use sprite::{RotatableSprite, Sprite};
use vek::{Extent2, Vec2};

use crate::graphics::Color;

/// Window size.
pub const SIZE: Extent2<usize> = Extent2::new(640, 360);
/// Updates per second of the update loop.
const UPDATES_PER_SECOND: u32 = 60;

/// Load the settings.
pub fn settings() -> AssetReadGuard<'static, Settings> {
    pixel_game_lib::asset("settings")
}

/// Load a sprite.
pub fn sprite(path: &str) -> AssetReadGuard<Sprite> {
    pixel_game_lib::asset(path)
}

/// Load a rotatable sprite.
pub fn rotatable_sprite(path: &str) -> AssetReadGuard<RotatableSprite> {
    pixel_game_lib::asset(path)
}

/// Load a font.
pub fn font(path: &str) -> AssetReadGuard<Font> {
    pixel_game_lib::asset(path)
}

fn main() -> Result<()> {
    // Construct the game
    let state = GameState::new();

    // Enable profiling server
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Run puffin HTTP profiling server
        let server_addr = format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT);
        let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
        println!("Puffin profiling server running at '{server_addr}', view with:\n\tpuffin_viewer --url 127.0.0.1:{}", puffin_http::DEFAULT_PORT);

        // Enable profiling
        puffin::set_scopes_on(true);
    }

    pixel_game_lib::window(
        state,
        WindowConfig {
            buffer_size: SIZE,
            title: "Castle Game".to_string(),
            updates_per_second: UPDATES_PER_SECOND,
            scaling: 4,
        },
        |g, input, mouse, dt| {
            puffin::profile_scope!("Update");

            // Update the game
            g.update(input, mouse.map(Vec2::as_), dt as f64);

            puffin::GlobalProfiler::lock().new_frame();

            // Exit when escape is pressed
            input.key_pressed(KeyCode::Escape)
        },
        |g, canvas, frame_time| {
            {
                puffin::profile_scope!("Clear pixels");
                canvas.fill(Color::SkyBlue.as_u32());
            }

            {
                puffin::profile_scope!("Render");

                // Draw the game
                g.render(canvas, frame_time as f64);
            }
        },
    )?;

    Ok(())
}

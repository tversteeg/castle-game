mod assets;
mod camera;
mod debug;
mod font;
mod game;
mod input;
mod math;
mod object;
mod physics;
mod projectile;
mod random;
mod sprite;
mod terrain;
mod timer;
mod unit;
mod window;

use std::sync::OnceLock;

use assets::Assets;
use assets_manager::{AssetGuard, Compound};
use font::Font;
use game::{GameState, Settings};
use miette::Result;
use sprite::{RotatableSprite, Sprite};
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
use vek::Extent2;

/// Window size.
pub const SIZE: Extent2<usize> = Extent2::new(360, 360);
/// Updates per second of the update loop.
const UPDATES_PER_SECOND: u32 = 60;

/// The assets as a 'static reference.
pub static ASSETS: OnceLock<Assets> = OnceLock::new();

/// Load an generic asset.
pub fn asset<T>(path: &str) -> AssetGuard<T>
where
    T: Compound,
{
    puffin::profile_function!();

    ASSETS
        .get()
        .expect("Asset handling not initialized yet")
        .asset(path)
}

/// Load the global settings.
pub fn settings() -> AssetGuard<'static, Settings> {
    ASSETS
        .get()
        .expect("Asset handling not initialized yet")
        .settings()
}

/// Load a sprite.
pub fn sprite(path: &str) -> AssetGuard<Sprite> {
    crate::asset(path)
}

/// Load a rotatable sprite.
pub fn rotatable_sprite(path: &str) -> AssetGuard<RotatableSprite> {
    crate::asset(path)
}

/// Load a font.
pub fn font(path: &str) -> AssetGuard<Font> {
    crate::asset(path)
}

async fn run() -> Result<()> {
    // Initialize the asset loader
    let assets = ASSETS.get_or_init(Assets::load);
    assets.enable_hot_reloading();

    // Construct the game
    let state = GameState::new();

    window::run(
        state,
        SIZE,
        UPDATES_PER_SECOND,
        |g, input, dt| {
            puffin::profile_scope!("Update");

            // Update the game
            g.update(input, dt);

            puffin::GlobalProfiler::lock().new_frame();
        },
        |g, buffer, frame_time| {
            {
                puffin::profile_scope!("Clear pixels");
                buffer.fill(0);
            }

            {
                puffin::profile_scope!("Render");

                // Draw the game
                g.render(buffer, frame_time);
            }
        },
    )
    .await?;

    Ok(())
}

/// Entry point starting either a WASM future or a Tokio runtime.
fn main() {
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(log::Level::Info).expect("error initializing logger");

        wasm_bindgen_futures::spawn_local(async { run().await.unwrap() });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Run puffin HTTP profiling server
        let server_addr = format!("0.0.0.0:{}", puffin_http::DEFAULT_PORT);
        let _puffin_server = puffin_http::Server::new(&server_addr).unwrap();
        println!("Puffin profiling server running at '{server_addr}', view with:\n\tpuffin_viewer --url 127.0.0.1:{}", puffin_http::DEFAULT_PORT);

        // Enable profiling
        puffin::set_scopes_on(true);

        let rt = Runtime::new().unwrap();
        rt.block_on(async { run().await.unwrap() });
    }
}

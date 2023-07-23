mod assets;
mod buffer;
mod camera;
mod font;
mod game;
mod input;
mod sprite;
mod terrain;
mod unit;
mod window;

use std::sync::OnceLock;

use assets::Assets;
use game::GameState;
use miette::Result;
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
use vek::Extent2;

/// Window size.
pub const SIZE: Extent2<usize> = Extent2::new(320, 180);
/// Frames per second of the render loop.
const FPS: u32 = 60;

/// The assets as a 'static reference.
static ASSETS: OnceLock<Assets> = OnceLock::new();

async fn run() -> Result<()> {
    // Initialize the assets once
    let assets = ASSETS.get_or_init(Assets::load);

    // Construct the game
    let state = GameState::new(assets);

    window::run(
        state,
        SIZE,
        FPS,
        |g, input| {
            // Update the game
            g.update(input);
        },
        |g, buffer| {
            buffer.fill(0);

            // Draw the game
            g.render(buffer);
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
        let rt = Runtime::new().unwrap();
        rt.block_on(async { run().await.unwrap() });
    }
}

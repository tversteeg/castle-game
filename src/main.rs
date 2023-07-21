mod font;
mod game;
mod input;
mod window;

use game::GameState;
use miette::{IntoDiagnostic, Result};
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;
use vek::Extent2;

pub const SIZE: Extent2<usize> = Extent2::new(320, 180);
const FPS: u32 = 60;

async fn run() -> Result<()> {
    // Construct the game
    let state = GameState::new();

    window::run(
        state,
        SIZE,
        FPS,
        |g, input| {
            // Update the game
            g.update();
        },
        |g, buffer| {
            buffer.fill(0);

            // Draw the game
            g.render(buffer, SIZE.into_tuple().into());
        },
    )
    .await?;

    Ok(())
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

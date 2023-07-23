use assets_manager::{AssetCache, AssetGuard};

use crate::{font::Font, sprite::Sprite};

/// All external data.
#[cfg(not(target_arch = "wasm32"))]
pub struct Assets(AssetCache<assets_manager::source::FileSystem>);
#[cfg(target_arch = "wasm32")]
pub struct Assets(AssetCache<assets_manager::source::Embedded<'static>>);

impl Assets {
    /// Construct the asset loader.
    ///
    /// Embeds all assets for the WASM target.
    pub fn load() -> Self {
        // Load the assets from disk, allows hot-reloading
        #[cfg(not(target_arch = "wasm32"))]
        let source = assets_manager::source::FileSystem::new("assets").unwrap();

        // Embed all assets into the binary
        #[cfg(target_arch = "wasm32")]
        let source =
            assets_manager::source::Embedded::from(assets_manager::source::embed!("assets"));

        let asset_cache = AssetCache::with_source(source);

        Self(asset_cache)
    }

    /// Load a sprite.
    pub fn sprite(&self, path: &str) -> AssetGuard<Sprite> {
        self.0.load_expect(path).read()
    }

    /// Load a font.
    pub fn font(&self, path: &str) -> AssetGuard<Font> {
        self.0.load_expect(path).read()
    }

    /// Hot reload from disk if applicable.
    pub fn enable_hot_reloading(&'static self) {
        self.0.enhance_hot_reloading();
    }
}

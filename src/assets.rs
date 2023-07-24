use assets_manager::{Asset, AssetCache, AssetGuard};

use crate::{
    font::Font,
    game::Settings,
    sprite::{RotatableSprite, Sprite},
};

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

    /// Load a rotatable sprite.
    pub fn rotatable_sprite(&self, path: &str) -> AssetGuard<RotatableSprite> {
        self.0.load_expect(path).read()
    }

    /// Load a font.
    pub fn font(&self, path: &str) -> AssetGuard<Font> {
        self.0.load_expect(path).read()
    }

    /// Load the settings.
    pub fn settings(&self) -> AssetGuard<Settings> {
        self.0.load_expect("settings").read()
    }

    /// Load an generic asset.
    pub fn asset<T>(&self, path: &str) -> AssetGuard<T>
    where
        T: Asset,
    {
        self.0.load_expect(path).read()
    }

    /// Hot reload from disk if applicable.
    pub fn enable_hot_reloading(&'static self) {
        self.0.enhance_hot_reloading();
    }
}

use crate::{font::Font, sprite::Sprite};

/// All external data.
pub struct Assets {
    /// Sprite for the default unit.
    pub unit_sprite: Sprite,
    /// Sprite for the default terrain.
    pub terrain_sprite: Sprite,
    /// Default font
    pub font: Font,
}

impl Assets {
    /// Load all assets immediately.
    pub fn load() -> Self {
        // Load the embedded font
        let font = Font::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/font/torus-sans.png"),
            (9, 9).into(),
        );

        // Load the embedded sprites
        let unit_sprite = Sprite::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/unit/spear-1.png"),
        );
        let terrain_sprite = Sprite::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/level/grass-1.png"),
        );

        Self {
            font,
            unit_sprite,
            terrain_sprite,
        }
    }
}

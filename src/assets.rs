use crate::{font::Font, sprite::Sprite};

/// All external data.
pub struct Assets {
    /// Sprite for the default unit body.
    pub unit_base_sprite: Sprite,
    /// Sprite for the default unit hands with spear.
    pub unit_weapon_sprite: Sprite,
    /// Sprite for the spear projectile.
    pub spear_projectile_sprite: Sprite,
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
        let unit_base_sprite = Sprite::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/unit/base-1.png"),
        );
        let unit_weapon_sprite = Sprite::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/unit/spear-hands-1.png"),
        );
        let spear_projectile_sprite = Sprite::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/projectile/spear-1.png"),
        );
        let terrain_sprite = Sprite::from_bytes(
            // Embed the image in the binary
            include_bytes!("../assets/level/grass-1.png"),
        );

        Self {
            font,
            unit_base_sprite,
            unit_weapon_sprite,
            spear_projectile_sprite,
            terrain_sprite,
        }
    }
}
